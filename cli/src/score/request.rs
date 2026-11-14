//! The verdict request of a check run (split from `run` at the E01 line,
//! plan v2.35 5C — the third split of that body): the measured facts,
//! the churn window when asked, the size facts and the provenance table,
//! then ce.toml's knob tables, assembled into ONE `wire::Request`. Rust
//! computes no policy here either: every table is a fact read off the
//! tree or a knob ce.toml declared. The literal names EVERY field: an
//! empty table is a legal wire fact, so a field added to `Request` must
//! stop the build here rather than ride empty in silence (the docdup
//! axis once read a constant zero that way; see `measure`).

use super::{
    Measured, Opts, baseline_of, churn_rows, doc_file_indices, knobs, located, provenance,
    size_facts, wire,
};
use crate::config::Config;
use anyhow::Result;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// The request over one measurement and the config `run` loaded.
pub(super) fn request(
    root: &Path,
    opts: &Opts,
    cfg: &Config,
    m: Measured,
) -> Result<wire::Request> {
    let idx: HashMap<&str, i64> = m.idx.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let (churn, cochange) = churn_rows(root, opts.days, &idx)?;
    let (continuous, at, judged_loc, classed) = size_facts(root, &opts.core, &m.anchors)?;
    let present = present_of(root, &continuous)?;
    let digest = cfg.knobs_digest();
    let baseline = baseline_of(opts, &continuous, &m.members, digest);
    let [ceilings, weights, thresholds, tolerance] = knob_tables(cfg)?;
    let (cont_spans, prose) = spans_and_prose(root, &at, &idx, &m.files)?;
    Ok(wire::Request {
        sim: m.sim,
        pos: m.pos,
        // the export surface (6.1.0): the candidates' RG10 guard has
        // no other way to learn which flank is a public API, and a
        // `delete` proposed for one is the exact false positive the
        // four-way dead code was split to prevent
        symbols: m.symbols,
        present: Some(present),
        // rides exactly at floor 1: the core requires it there and
        // refuses it anywhere else (O59)
        cycle_self_loops: (cfg.graph.scc_floor == Some(1)).then_some(m.self_loops),
        churn,
        cochange,
        continuous,
        classed,
        class_knobs: knobs::class_knob_rows(&cfg.rules),
        knobs_digest: digest,
        discrete: m.members,
        baseline,
        floor: opts.floor,
        ceilings,
        weights,
        thresholds,
        tolerance,
        // the dedup pair is `ce dedup --check`'s leg alone (P2): this road stays byte-identical
        dedup: None,
        dedup_distinct: Vec::new(),
        dedup_min_distinct: None,
        judged_loc,
        doc_files: doc_file_indices(&m.files),
        extent: (cont_spans, m.dup_spans, m.ref_lines, prose),
        files: m.files,
        dead: m.dead,
    })
}

/// ce.toml's four knob tables on the request (ADR-008: the first step
/// sent the ceilings; P4 added weights, thresholds and tolerance through
/// score::knobs' declared tables), in that order.
fn knob_tables(cfg: &Config) -> Result<[wire::KnobTable; 4]> {
    Ok([
        knobs::ceiling_rows(&cfg.thresholds, &cfg.score),
        knobs::weight_rows(&cfg.score)?,
        knobs::threshold_rows(&cfg.score, cfg.graph.scc_floor),
        knobs::tolerance_rows(&cfg.score),
    ])
}

/// `contSpans` and `proseLines`, in that order.
type SpansAndProse = (Vec<[i64; 3]>, Vec<[i64; 2]>);

/// Two of the extent law's four tables (v2.35; score::located), off the
/// continuous rows' homes and spans: `contSpans`, aligned with those rows,
/// and `proseLines`, each prose-only file's line count.
fn spans_and_prose(
    root: &Path,
    (homes, spans): &(Vec<String>, Vec<(i64, i64)>),
    idx: &HashMap<&str, i64>,
    files: &[String],
) -> Result<SpansAndProse> {
    let cont = located::cont_spans(homes, spans, idx);
    Ok((cont, located::prose_lines(root, files, homes)?))
}

/// The provenance table (6.4.0, O40): every file entity on disk under
/// the scope that owns no row this run — the core answers which
/// committed rows they explain (`ratchet.dropped`).
fn present_of(root: &Path, continuous: &[[u64; 4]]) -> Result<Vec<u64>> {
    let measured: BTreeSet<u64> = continuous
        .iter()
        .filter(|r| r[1] == 0)
        .map(|r| r[0])
        .collect();
    provenance::present(root, &measured)
}
