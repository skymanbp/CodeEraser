//! T3-universe instrument shared surface (M5-3b, design vol.3 §9.2):
//! the pre-registered sizeband constants, the per-file row derivation
//! and the materialized-index leg — ONE binding for the generator
//! (eval_t3_universe.rs) and its self-repo drift gate, so the frozen
//! doc and the recheck can never diverge on what a row means. Facts
//! flow through the SAME throats the product writer uses:
//! `sites::detect_with_units` + `units::with_nth` for symbols (the
//! store::refresh_graph pair), `unitcache::unit_facts` for units.

use codeeraser::dedup::{refreshed_index, unitcache};
use codeeraser::fourclass::units;
use codeeraser::graph::sites;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Pre-registered sizeband bounds in NAMED nodes (design §9.2:
/// S:24-60, M:61-200, L:>200; below 24 is the below_floor ledger —
/// structurally invisible to the candidate pass, so candidate recall
/// is an upper bound and every doc's method says so). One Rust
/// binding; the doc constants and the band function both read it.
pub const T3_BANDS: [(&str, i64); 3] = [("s", 60), ("m", 200), ("l", i64::MAX)];
pub const T3_FLOOR_NODES: i64 = 24;

/// The frozen constants object every t3-universe doc embeds.
pub fn t3_constants() -> Value {
    json!({
        "floor_nodes": T3_FLOOR_NODES,
        "s_max_nodes": T3_BANDS[0].1,
        "m_max_nodes": T3_BANDS[1].1,
    })
}

/// The band of one unit's named-node count.
pub fn band_of(nodes: i64) -> &'static str {
    if nodes < T3_FLOOR_NODES {
        return "below_floor";
    }
    T3_BANDS
        .iter()
        .find(|(_, max)| nodes <= *max)
        .expect("l is unbounded")
        .0
}

/// One t3-universe row: content identity plus the symbols and unit
/// universes of the text. The (key, nth) identity agreement between
/// the two views (the pure analog of `ce clone --units`'s
/// identity-orphans assertion) and per-file identity uniqueness (the
/// UNIQUE(file_id, key, nth) precondition) are asserted per corpus
/// file, at derivation time.
pub fn t3_row(path: &str, code: &str, content: &str) -> Value {
    let lang = super::lang_of(code);
    let (_, segments) = sites::detect_with_units(content, lang);
    let symbols: BTreeSet<(&str, i64)> = units::with_nth(&segments)
        .into_iter()
        .map(|(u, nth)| (u.key.as_str(), nth))
        .collect();
    assert_eq!(
        symbols.len(),
        segments.len(),
        "{path}: duplicate (key, nth) identity"
    );
    let facts = unitcache::unit_facts(content, lang);
    let mut bands: BTreeMap<&str, u64> = [("below_floor", 0), ("s", 0), ("m", 0), ("l", 0)].into();
    for f in &facts {
        assert!(
            symbols.contains(&(f.key.as_str(), f.nth)),
            "{path}: unit {}#{} missing its symbols identity",
            f.key,
            f.nth
        );
        *bands.get_mut(band_of(f.nodes)).expect("band") += 1;
    }
    json!({"path": path, "sha256": super::content_sha(content), "lang": code,
           "symbols": symbols.len(), "units": facts.len(), "bands": bands})
}

/// Re-derivable from the rows alone — the CI gate re-runs this exact
/// function (the G1 discipline: generator and gate share one scorer).
/// Per-file conservation (units == Σ bands) is asserted while
/// summing, so a row whose bands leak units can never freeze.
pub fn t3_summarize(files: &[Value]) -> Value {
    let mut symbols_by: BTreeMap<String, u64> = BTreeMap::new();
    let mut units_by: BTreeMap<String, u64> = BTreeMap::new();
    let mut bands_by: BTreeMap<String, u64> = BTreeMap::new();
    let (mut symbols, mut units) = (0u64, 0u64);
    for f in files {
        let lang = f["lang"].as_str().expect("lang");
        let s = f["symbols"].as_u64().expect("symbols");
        let u = f["units"].as_u64().expect("units");
        super::tally_add(&mut symbols_by, lang, s);
        super::tally_add(&mut units_by, lang, u);
        symbols += s;
        units += u;
        let mut in_bands = 0;
        for (band, n) in f["bands"].as_object().expect("bands") {
            let n = n.as_u64().expect("count");
            super::tally_add(&mut bands_by, &format!("{lang}/{band}"), n);
            in_bands += n;
        }
        assert_eq!(u, in_bands, "{}: units escape the bands", f["path"]);
    }
    json!({"files": files.len(), "total_symbols": symbols, "symbols_by": symbols_by,
           "total_units": units, "units_by": units_by, "bands_by": bands_by})
}

/// The product-index leg of the generator: materialize the in-scope
/// tree, run the real indexer twice (cold, then warm), and let the
/// REAL tables prove what the pure derivation asserted — the
/// UNIQUE(file_id, key, nth) inserts land collision-free on this
/// corpus and the two persisted views agree (identity_orphans == 0).
/// The timings are the honest fourth-parse cost (F11) for the
/// PERF-BUDGET ledger — admissible only from a release build, and
/// labeled with the build profile so a debug number cannot pass as
/// one.
pub fn index_materialized(corpus: &str, walked: &[super::WalkedFile], doc: &Value) {
    let root = materialize_tree("univ", corpus, walked);
    let t0 = std::time::Instant::now();
    let (idx, _) = refreshed_index(&root, None).expect("cold index");
    let cold = t0.elapsed().as_millis();
    assert_eq!(
        unitcache::identity_orphans(&idx).expect("orphans"),
        0,
        "{corpus}: unitsig rows missing their symbols identity"
    );
    check_indexed_subset(corpus, &root, walked, doc, &idx);
    drop(idx);
    let t1 = std::time::Instant::now();
    drop(refreshed_index(&root, None).expect("warm index"));
    let warm = t1.elapsed().as_millis();
    let profile = if cfg!(debug_assertions) {
        "debug build — NOT admissible for PERF-BUDGET"
    } else {
        "release"
    };
    println!(
        "PERF t3-universe {corpus}: cold {cold} ms, warm {warm} ms, {} files ({profile})",
        walked.len()
    );
    std::fs::remove_dir_all(&root).expect("drop temp tree");
}

/// Materialize one walked in-scope tree under the OS temp dir — the
/// shared opening of the universe instrument's product-index leg and
/// the candidate instrument (one tree bytes, two consumers).
pub fn materialize_tree(
    tag: &str,
    corpus: &str,
    walked: &[super::WalkedFile],
) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("ce-t3-{tag}-{corpus}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("clean temp tree");
    }
    for (path, _, text) in walked {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        std::fs::write(&file, text.as_bytes()).expect("materialize");
    }
    root
}

/// Indexed ⊆ walked (an indexed phantom is a scope leak), every
/// walked-but-unindexed path is provably product-invisible (hidden,
/// or excluded by the product's own scope test — the instrument scope
/// is deliberately wider: the frozen universe keeps .github/*.md
/// while the product walk hides dot paths), and every indexed file's
/// cached unit count equals its frozen row.
fn check_indexed_subset(
    corpus: &str,
    root: &Path,
    walked: &[super::WalkedFile],
    doc: &Value,
    idx: &codeeraser::dedup::index::Index,
) {
    let (indexed, _, _) = codeeraser::graph::load::graph_rows(idx).expect("graph rows");
    let walked_set: BTreeSet<&str> = walked.iter().map(|(p, _, _)| p.as_str()).collect();
    for path in &indexed {
        assert!(
            walked_set.contains(path.as_str()),
            "{corpus}: indexed phantom {path}"
        );
    }
    let indexed_set: BTreeSet<&str> = indexed.iter().map(String::as_str).collect();
    for (path, _, _) in walked {
        let visible = indexed_set.contains(path.as_str())
            || path.starts_with('.')
            || path.contains("/.")
            || !codeeraser::scan::walk::in_scope(root, &root.join(path), &[]);
        assert!(
            visible,
            "{corpus}: {path} silently missing from the product index"
        );
    }
    let mut per_file: BTreeMap<String, u64> = BTreeMap::new();
    for u in unitcache::unit_rows(idx).expect("unit rows") {
        *per_file.entry(u.path).or_insert(0) += 1;
    }
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        if !indexed_set.contains(path) {
            continue;
        }
        assert_eq!(
            per_file.get(path).copied().unwrap_or(0),
            row["units"].as_u64().expect("units"),
            "{corpus}: {path}: cached units != frozen units"
        );
    }
}
