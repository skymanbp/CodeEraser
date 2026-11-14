//! Content probe (M3 PreToolUse cheap gate, ADR-004): given the text
//! about to be written, find verified T1/T2 matches against the
//! project index. Deliberately CHEAP: no index refresh, no walk. The
//! index SELECTS the candidate files (the ones holding a fingerprint
//! of the content); each candidate is then matched by the fingerprints
//! of its CURRENT token stream, which the probe reads anyway (v2.35
//! guard lane, GD5): a candidate edited since the last refresh used to
//! be extended at its stale indexed offsets, and its twin went unseen
//! until the Stop audit. On an unchanged file the current fingerprints
//! ARE the indexed ones (the same winnowing over the same stream), so a
//! fresh index answers exactly as before.

use super::index::Index;
use super::pairs::{self, Filter, Streams};
use super::{Params, tokens, winnow};
use crate::scan::lang::Lang;
use anyhow::Result;
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
    /// Where the run starts in the PROBED content's token stream (v2.35
    /// GD4, additive): the hook's relocation stage counts that very run
    /// in HEAD and in the tree the write leaves. With several copies, the
    /// first one's start (every copy holds the same token sequence).
    pub at: usize,
    /// How many copies of this source run the PROBED content holds:
    /// the most pairwise disjoint [start, start + tokens) intervals
    /// among the starts whose maximal extension is exactly this run
    /// (release 1.10.0, additive in daemon 2.4.0). The guard subtracts
    /// in these units, so a second copy written beside a carried one is
    /// new duplication.
    pub copies: usize,
}

/// What is about to be written: destination (excluded from matching —
/// the index still holds its pre-edit version), bytes, and language.
pub struct Target<'a> {
    pub rel: &'a str,
    pub content: &'a [u8],
    pub lang: Lang,
}

/// Probe the target content against the project index.
pub fn probe(
    idx: &Index,
    root: &Path,
    target: Target<'_>,
    p: Params,
    f: Filter,
) -> Result<Vec<Match>> {
    let Target {
        rel: rel_self,
        content,
        lang,
    } = target;
    let snippet = tokens::stream(content, lang)?;
    let hashes: Vec<u64> = snippet.iter().map(|t| t.hash).collect();
    let fps = winnow::fingerprints(&hashes, p);
    let wanted: Vec<u64> = fps.iter().map(|fp| fp.hash).collect();
    let files: BTreeSet<String> = idx
        .instances_for_hashes(&wanted)?
        .into_iter()
        .map(|i| i.file)
        .filter(|file| file != rel_self)
        .collect();
    let streams = load_candidate_streams(root, &files)?;
    let instances = current_instances(&streams, &wanted.into_iter().collect(), p);
    Ok(collect_matches(&snippet, &fps, &instances, &streams, f))
}

/// The candidates' fingerprints as their current streams give them,
/// kept where the content holds the same hash — what the index would
/// hold for each file after a refresh, without writing it.
fn current_instances(
    streams: &Streams,
    wanted: &BTreeSet<u64>,
    p: Params,
) -> Vec<super::index::Instance> {
    let mut out = Vec::new();
    for (file, stream) in streams {
        let hashes: Vec<u64> = stream.iter().map(|t| t.hash).collect();
        for fp in winnow::fingerprints(&hashes, p) {
            if wanted.contains(&fp.hash) {
                out.push(super::index::Instance {
                    hash: fp.hash,
                    file: file.clone(),
                    start_tok: fp.start,
                    start_line: stream[fp.start].start_line,
                    end_line: stream[fp.start + p.kgram - 1].end_line,
                });
            }
        }
    }
    out
}

fn load_candidate_streams(root: &Path, files: &BTreeSet<String>) -> Result<Streams> {
    let mut out = Streams::new();
    for rel in files {
        let Some((path, lang)) = super::walkidx::lang_path(root, rel) else {
            continue;
        };
        // a candidate that vanished between indexing and this probe
        // (delete/rename race — hit by the FPR replay on a real
        // rename commit) is stale, not fatal: its anchors simply
        // find no stream, same as the stale-offset skip
        let Ok(src) = std::fs::read(&path) else {
            continue;
        };
        out.insert(rel.to_string(), tokens::stream(&src, lang)?);
    }
    Ok(out)
}

/// Extend every (snippet anchor, indexed instance) hash pair and keep
/// verified maximal runs above the filter: ONE match per file-side run
/// `(file, b0, len)`, pushed at its first hit, carrying the set of
/// probed-side starts whose extension is exactly that run as `copies`
/// (`disjoint`) and its smallest start as `at`.
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
    // each run's place in `out` and its probed-side starts
    let mut runs: BTreeMap<(String, usize, usize), (usize, BTreeSet<usize>)> = BTreeMap::new();
    let mut out = Vec::new();
    for fp in fps {
        for inst in by_hash.get(&fp.hash).map(Vec::as_slice).unwrap_or(&[]) {
            let Some(stream) = streams.get(&inst.file) else {
                continue;
            };
            let Some((a0, b0, len, distinct)) = verified(snippet, fp.start, stream, inst, f) else {
                continue;
            };
            let (_, starts) = runs
                .entry((inst.file.clone(), b0, len))
                .or_insert_with(|| (out.len(), BTreeSet::new()));
            if starts.insert(a0) && starts.len() == 1 {
                out.push(Match {
                    file: inst.file.clone(),
                    start_line: stream[b0].start_line,
                    end_line: stream[b0 + len - 1].end_line,
                    tokens: len,
                    distinct,
                    at: a0,
                    copies: 1,
                });
            }
        }
    }
    for ((_, _, len), (i, starts)) in &runs {
        out[*i].at = starts.first().copied().unwrap_or(out[*i].at);
        out[*i].copies = disjoint(starts, *len);
    }
    out.sort_by(|x, y| y.tokens.cmp(&x.tokens).then(x.file.cmp(&y.file)));
    out
}

/// One hash pair's maximal verified run, kept only above the filter:
/// the probed-side start, the file-side start, the length and the
/// run's distinct token count.
fn verified(
    snippet: &[tokens::Token],
    at: usize,
    stream: &[tokens::Token],
    inst: &super::index::Instance,
    f: Filter,
) -> Option<(usize, usize, usize, usize)> {
    let (a0, b0, len) = pairs::extend(snippet, at, stream, inst.start_tok, false)?;
    let distinct = snippet[a0..a0 + len]
        .iter()
        .map(|t| t.hash)
        .collect::<BTreeSet<_>>()
        .len();
    (len >= f.min_tokens && distinct >= f.min_distinct).then_some((a0, b0, len, distinct))
}

/// The most pairwise disjoint intervals [a, a + len) among `starts`:
/// one length, so taking them greedily in ascending start (a start
/// once it is at least the last taken start + len) is a largest
/// disjoint family — overlapping starts in periodic text count once.
fn disjoint(starts: &BTreeSet<usize>, len: usize) -> usize {
    let mut last: Option<usize> = None;
    let mut n = 0;
    for &a in starts {
        if last.is_none_or(|l| a >= l + len) {
            last = Some(a);
            n += 1;
        }
    }
    n
}
