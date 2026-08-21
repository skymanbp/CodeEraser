//! The per-request reply builders (split from server.rs when the
//! 1.1.0 auth gate pushed it past the 300-line dogfood wall):
//! everything that turns one authed request into one Response.

use super::super::coldstart;
use super::super::proto::Response;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(super) fn dedup_reply(
    root: &Path,
    min_tokens: Option<usize>,
    min_distinct: Option<usize>,
) -> Response {
    match run_dedup(root, min_tokens, min_distinct) {
        Ok(report) => {
            coldstart::mark_ready();
            Response::DedupReport { report }
        }
        Err(e) => Response::Error {
            message: format!("{e:#}"),
        },
    }
}

pub(super) fn probe_reply(root: &Path, file_path: &str, content: &str) -> Response {
    if !coldstart::index_ready(root) {
        return Response::Error {
            message: "index cold start: first build in progress".into(),
        };
    }
    let t0 = Instant::now();
    match run_probe(root, file_path, content) {
        Ok(matches) => Response::ProbeReport {
            matches,
            elapsed_ms: t0.elapsed().as_millis() as u64,
        },
        Err(e) => Response::Error {
            message: format!("{e:#}"),
        },
    }
}

/// Cheap by design (ADR-004): opens the index read-path only, never
/// refreshes; unknown language probes report no matches.
fn run_probe(root: &Path, file_path: &str, content: &str) -> Result<serde_json::Value> {
    use crate::dedup::{Params, index::Index, pairs, probe};
    use crate::scan::lang::Lang;
    let path = Path::new(file_path);
    let Some(lang) = Lang::from_path(path) else {
        return Ok(serde_json::json!([]));
    };
    if lang.grammar().is_none() {
        return Ok(serde_json::json!([]));
    }
    // Same containment authority as the FourClass leg: the path rides
    // a token-gated but root-scoped socket, and one outside the served
    // root is not ours to judge — its raw spelling would also become
    // a self-exclusion key that excludes nothing.
    let Some(joined) = crate::scan::walk::contained(root, file_path) else {
        return Ok(serde_json::json!([]));
    };
    // The SAME exclusion model the budget rule honours. This leg had
    // only containment, so a write into vendor/ or migrations/, or to
    // a path a human deliberately listed in .ceignore, could be DENIED
    // as a duplicate while the sibling rule on the identical write
    // correctly stayed silent — and the FPR record the deny tier rests
    // on replayed no excluded-path targets at all.
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    if !crate::scan::walk::in_scope(root, &joined, &cfg.exclude) {
        return Ok(serde_json::json!([]));
    }
    // canonicalize before stripping: `root` is canonical (serve()),
    // and on Windows that is the \\?\ verbatim form — a plain
    // absolute file_path never strip-matches it, which silently
    // killed probe self-exclusion (caught by the observe-feed golden
    // diverging across CI platforms). A not-yet-existing file can't
    // canonicalize — and can't be in the index either, so the raw
    // fallback is safe. The SPELLING then goes through the one
    // rel_str throat (M5-close review: this was the third copy).
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let rel = crate::scan::walk::rel_str(root, &canon);
    let p = Params::default();
    let idx = Index::open(&root.join(".ce/index.db"), p)?;
    let f = pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    };
    let target = probe::Target {
        rel: &rel,
        content: content.as_bytes(),
        lang,
    };
    let matches = probe::probe(&idx, root, target, p, f)?;
    Ok(serde_json::to_value(matches)?)
}

fn run_dedup(
    root: &Path,
    min_tokens: Option<usize>,
    min_distinct: Option<usize>,
) -> Result<serde_json::Value> {
    let db: Option<PathBuf> = None; // daemon always uses <root>/.ce/index.db
    let (found, summary) = crate::dedup::analyze(root, db, min_tokens, min_distinct)?;
    crate::dedup::report_json(&found, &summary)
}
