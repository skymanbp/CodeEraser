//! M5-3j gate-migration contract (F4): the interim CI awk gate
//! (300-line ceiling over core/**/*.hs) is retired in favor of
//! per-file ratchet ceilings in ce-baseline.json, judged by
//! `ce check`. Replacement-not-weaker is asserted PER FILE, on the
//! retired gate's own universe (a recursive walk of core/app +
//! core/test — deliberately independent of the product's exclusion
//! model, so a walk regression cannot silently shrink the contract):
//! the product emits the file's size row, the committed baseline
//! covers it, and its tolerated ceiling stays within the awk red
//! line. New unbaselined entities bootstrap silently in the ratchet
//! (Ratchet.hs, by design) — the coverage leg here is what closes
//! that door for core files.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Mirror of CE.Verdict.Cost tolNum/tolDen/tolAbs (102/100/10); the
/// formula itself is pinned core-side by Spec.hs costModel's two
/// branch assertions. Drift here fails this gate loudly, either way.
fn tolerated(c: u64) -> u64 {
    (c * 102 / 100).max(c + 10)
}

fn hs_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read core dir") {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            hs_under(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("hs") {
            out.push(p);
        }
    }
}

/// The committed baseline's file-size ceilings (metricCode 0).
fn baseline_ceilings(root: &Path) -> HashMap<u64, u64> {
    let text = fs::read_to_string(root.join("ce-baseline.json")).expect("committed baseline");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("baseline json");
    doc["continuous"]
        .as_array()
        .expect("continuous table")
        .iter()
        .filter(|row| row[1].as_u64() == Some(0))
        .map(|row| {
            (
                row[0].as_u64().expect("entity"),
                row[2].as_u64().expect("value"),
            )
        })
        .collect()
}

#[test]
fn ratchet_ceilings_dominate_the_retired_awk_gate_per_file() {
    let root = Path::new("..");
    let mut files = Vec::new();
    hs_under(&root.join("core/app"), &mut files);
    hs_under(&root.join("core/test"), &mut files);
    assert!(
        files.len() >= 20,
        "vacuous universe: only {} .hs files found",
        files.len()
    );
    let current: HashMap<u64, u64> = codeeraser::score::continuous_rows(root)
        .expect("continuous assembly")
        .into_iter()
        .filter(|r| r[1] == 0)
        .map(|r| (r[0], r[2]))
        .collect();
    let committed = baseline_ceilings(root);
    for f in &files {
        let rel = codeeraser::scan::walk::rel_str(root, f);
        let entity = codeeraser::score::baseline::file_entity(&rel);
        let loc = codeeraser::scan::metrics::size::total_lines(&fs::read(f).expect("read .hs"));
        assert_eq!(
            current.get(&entity),
            Some(&(loc as u64)),
            "{rel}: the product emits no size row for this file"
        );
        let ceiling = *committed.get(&entity).unwrap_or_else(|| {
            panic!("{rel}: not in the committed baseline — run ce baseline and commit the file")
        });
        assert!(
            tolerated(ceiling) <= 300,
            "{rel}: tolerated({ceiling}) = {} exceeds the retired awk red line of 300 — split the file (E01)",
            tolerated(ceiling)
        );
    }
}

/// RM20: after the 3j split, EVAL-SET.md's 300 turns from a warning
/// line into a hard ceiling — the committed baseline must never
/// record it above the E01 line (new records land in
/// EVAL-SET-M5-3.md, frozen history is never compressed).
#[test]
fn eval_set_ceiling_is_pinned_at_the_e01_line() {
    let committed = baseline_ceilings(Path::new(".."));
    let entity = codeeraser::score::baseline::file_entity("docs/EVAL-SET.md");
    let ceiling = committed.get(&entity).expect("EVAL-SET.md in baseline");
    assert!(
        *ceiling <= 300,
        "EVAL-SET.md ceiling {ceiling} above the E01 line"
    );
}

/// Two gates over one rule drift silently — the migration deletes
/// the awk step, and its absence is part of the contract (3j).
#[test]
fn the_awk_gate_is_retired_exactly_once() {
    let ci = fs::read_to_string("../.github/workflows/ci.yml").expect("ci.yml");
    assert!(
        !ci.contains("awk"),
        "ci.yml still carries an awk gate — per-file ratchet ceilings replaced it (3j); \
         two gates over one rule drift silently"
    );
}
