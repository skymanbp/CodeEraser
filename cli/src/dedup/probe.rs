//! Content probe (M3 PreToolUse cheap gate, ADR-004): given the text
//! about to be written, find verified T1/T2 matches against the
//! project index. Deliberately CHEAP: no index refresh, no walk —
//! the index may lag the working tree (deep judgment runs at
//! PostToolUse/Stop); a stale anchor is skipped, never a panic.

use super::index::Index;
use super::pairs::{self, Filter, Streams};
use super::{Params, tokens, winnow};
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// A verified match between the probed content and an indexed file.
#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    /// Exact verified common-run length in normalized tokens.
    pub tokens: usize,
    pub distinct: usize,
}

/// Probe `content` (destined for `rel_self`, which is excluded — the
/// index still holds its pre-edit version) against the project index.
pub fn probe(
    idx: &Index,
    root: &Path,
    rel_self: &str,
    content: &[u8],
    lang: Lang,
    p: Params,
    f: Filter,
) -> Result<Vec<Match>> {
    let snippet = tokens::stream(content, lang)?;
    let hashes: Vec<u64> = snippet.iter().map(|t| t.hash).collect();
    let fps = winnow::fingerprints(&hashes, p);
    let wanted: Vec<u64> = fps.iter().map(|fp| fp.hash).collect();
    let instances: Vec<_> = idx
        .instances_for_hashes(&wanted)?
        .into_iter()
        .filter(|i| i.file != rel_self)
        .collect();
    let streams = load_candidate_streams(root, &instances)?;
    Ok(collect_matches(&snippet, &fps, &instances, &streams, f))
}

fn load_candidate_streams(root: &Path, instances: &[super::index::Instance]) -> Result<Streams> {
    let files: BTreeSet<&str> = instances.iter().map(|i| i.file.as_str()).collect();
    let mut out = Streams::new();
    for rel in files {
        let path = root.join(rel);
        let Some(lang) = Lang::from_path(&path) else {
            continue;
        };
        let src = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        out.insert(rel.to_string(), tokens::stream(&src, lang)?);
    }
    Ok(out)
}

/// Extend every (snippet anchor, indexed instance) hash pair and keep
/// verified maximal runs above the filter; dedupe by file-side span.
fn collect_matches(
    snippet: &[tokens::Token],
    fps: &[winnow::Fingerprint],
    instances: &[super::index::Instance],
    streams: &Streams,
    f: Filter,
) -> Vec<Match> {
    let mut by_hash: BTreeMap<u64, Vec<&super::index::Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    let mut runs: BTreeSet<(String, usize, usize)> = BTreeSet::new();
    let mut out = Vec::new();
    for fp in fps {
        for inst in by_hash.get(&fp.hash).map(Vec::as_slice).unwrap_or(&[]) {
            let Some(stream) = streams.get(&inst.file) else {
                continue;
            };
            if inst.start_tok >= stream.len() {
                continue; // index lags the working tree — deep pass will catch up
            }
            let Some((a0, b0, len)) =
                pairs::extend(snippet, fp.start, stream, inst.start_tok, false)
            else {
                continue;
            };
            let distinct = snippet[a0..a0 + len]
                .iter()
                .map(|t| t.hash)
                .collect::<BTreeSet<_>>()
                .len();
            if len < f.min_tokens
                || distinct < f.min_distinct
                || !runs.insert((inst.file.clone(), b0, len))
            {
                continue;
            }
            out.push(Match {
                file: inst.file.clone(),
                start_line: stream[b0].start_line,
                end_line: stream[b0 + len - 1].end_line,
                tokens: len,
                distinct,
            });
        }
    }
    out.sort_by(|x, y| y.tokens.cmp(&x.tokens).then(x.file.cmp(&y.file)));
    out
}
