//! Review data + mechanical partition machinery for the commit-slice
//! ground truth (see eval_commit_labels.rs for the semantics). The
//! two const tables ARE the per-item review record (2026-08-10):
//! every entry was verified against the raw diff it describes.
//!
//! Compiled independently by eval_commit_labels and eval_l2, each
//! using a subset — the unused remainder is expected.
#![allow(dead_code)]

use crate::eval_support::{BodyLine, commit_color_diff, walk_color_diff};
use codeeraser::fourclass::significant;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

/// Reviewed content-coincidence corrections: (sha prefix, file,
/// added-side, lines, mechanism).
const CORRECTIONS: [(&str, &str, bool, u64, &str); 5] = [
    (
        "da8eb210b",
        "cli/tests/guard_hook.rs",
        false,
        1,
        "removal was an inline-into-helper rewrite in the same file \
         (helper takes `dir`, not `&dir`); the matching addition in \
         mcp_precommit.rs is an unrelated new assertion",
    ),
    (
        "da8eb210b",
        "cli/tests/mcp_precommit.rs",
        true,
        1,
        "freshly written assertion; content coincides with the line \
         removed from guard_hook.rs",
    ),
    (
        "8d6b237a5",
        "cli/src/dedup/mod.rs",
        false,
        2,
        "run() params became pub struct fields in the same file \
         (textually changed by `pub`); git cross-paired the removals \
         to main.rs's own within-file enum-to-struct restructure",
    ),
    (
        "8d6b237a5",
        "cli/tests/dedup_check.rs",
        true,
        1,
        "file created in this commit; seed line freshly authored, \
         content coincides with removals elsewhere",
    ),
    (
        "4822d04ff",
        "cli/tests/daemon_e2e.rs",
        true,
        1,
        "freshly authored seed line; content coincides with removals \
         from other files",
    ),
];

/// Reviewed relocation targets: (sha prefix, receiving file,
/// comma-joined unit names). Qualitative GT for the unit-attribution
/// dimension; bodies may be adapted in flight (the quantitative gate
/// stays line-level). A "~" prefix marks a unit reviewed as
/// adapted-in-flight — zero line-identical body lines survive
/// (verified against the raw diff: the 2f40f22b8 helpers were
/// generalized while moving, e.g. write_doc gained parameters), so
/// line-level attribution is structurally impossible there and the
/// register records that instead of pretending.
const RELOCATED_UNITS: [(&str, &str, &str); 11] = [
    ("384d4c790", "cli/src/dedup/tokens.rs", "stream"),
    (
        "4822d04ff",
        "cli/tests/common/mod.rs",
        "tmp,rust_fn,git,seed_sources,build_index,run_hook,\
         spawn_daemon_ready,last_observe,shutdown_daemon,wait_exit",
    ),
    ("8d6b237a5", "cli/tests/common/mod.rs", "seed_clone_pair"),
    (
        "a3cf54d1f",
        "cli/tests/common/mod.rs",
        "seed_git_clone_repo",
    ),
    (
        "6d8824e20",
        "cli/src/hookio.rs",
        "read_envelope,observe_append",
    ),
    ("1ef9cdf60", "cli/tests/common/mod.rs", "parse"),
    ("da8eb210b", "cli/tests/common/mod.rs", "corrupt_index"),
    (
        "71b607d56",
        "cli/tests/common/mod.rs",
        "golden_path,assert_matches_golden,pretooluse_envelope,\
         stop_envelope,silent_hook_observe",
    ),
    (
        "822e7182f",
        "cli/src/daemon/coldstart.rs",
        "indexed_file_count,mark_ready,init,index_ready",
    ),
    (
        "f84aaa3c8",
        "cli/tests/eval_support/mod.rs",
        "CLASSES,load,by_id,numstat",
    ),
    (
        "2f40f22b8",
        "cli/tests/eval_support/mod.rs",
        "~out_dir,read_sample,~labeling_rows,finish_rows,~write_doc",
    ),
];

pub type PerFile = HashMap<String, u64>;

pub fn total(m: &PerFile) -> u64 {
    m.values().sum()
}

#[derive(Default)]
pub struct SideBuckets {
    pub nonsig: PerFile,
    pub cross: PerFile,
    /// Line identities behind `cross`, per file (attack review F2:
    /// counts alone cannot see a same-count substitution). A file
    /// with a reviewed correction is REMOVED from this map — the
    /// corrected lines' identities were never archived, so only its
    /// count (plus the coincidence-exact gate) remains authoritative.
    pub cross_lines: HashMap<String, Vec<usize>>,
    pub within: u64,
}

/// Moved-line partition of one commit, per direction (out = removed
/// side, into = added side).
#[derive(Default)]
pub struct Partition {
    pub out: SideBuckets,
    pub into: SideBuckets,
}

impl Partition {
    fn side_mut(&mut self, added: bool) -> &mut SideBuckets {
        if added { &mut self.into } else { &mut self.out }
    }
}

/// Trimmed content → files carrying it, per side, significant moved
/// lines only (a significant line's partner is significant too).
type SigSets<'a> = HashMap<&'a str, HashSet<&'a str>>;
fn sig_sets(lines: &[BodyLine]) -> (SigSets<'_>, SigSets<'_>) {
    let (mut removed, mut added) = (SigSets::new(), SigSets::new());
    for l in lines {
        if !significant(&l.content) {
            continue;
        }
        let map = if l.added { &mut added } else { &mut removed };
        map.entry(l.content.trim())
            .or_default()
            .insert(l.own_path());
    }
    (removed, added)
}

/// Mechanical layers 1+2 for one commit: significance filter, then
/// cross/within partition of the significant moved lines.
pub fn partition(sha: &str) -> Partition {
    let base = format!("{sha}^");
    let raw = commit_color_diff(&base, sha);
    let lines: Vec<BodyLine> = walk_color_diff(&raw)
        .into_iter()
        .filter(|l| l.moved)
        .collect();
    let (removed, added) = sig_sets(&lines);
    let mut p = Partition::default();
    for l in &lines {
        let own = l.own_path();
        let opposite = if l.added { &removed } else { &added };
        let side = p.side_mut(l.added);
        if !significant(&l.content) {
            *side.nonsig.entry(own.into()).or_default() += 1;
        } else {
            let files = opposite
                .get(l.content.trim())
                .unwrap_or_else(|| panic!("{sha}: unpaired moved line {:?}", l.content));
            if files.contains(own) {
                side.within += 1;
            } else {
                *side.cross.entry(own.into()).or_default() += 1;
                side.cross_lines.entry(own.into()).or_default().push(l.line);
            }
        }
    }
    p
}

/// Layer 3: move the reviewed coincidence lines out of the mechanical
/// cross bucket (they all land there — a coincidence has no
/// within-file partner). Returns the applied entries for the record.
pub fn apply_corrections(sha: &str, p: &mut Partition) -> Vec<Value> {
    let mut applied = Vec::new();
    for (pre, file, added, n, why) in CORRECTIONS {
        if !sha.starts_with(pre) {
            continue;
        }
        let side_b = p.side_mut(added);
        let c = side_b
            .cross
            .get_mut(file)
            .unwrap_or_else(|| panic!("{sha}: correction target {file} not cross"));
        assert!(*c >= n, "{sha}: correction exceeds cross count of {file}");
        *c -= n;
        if *c == 0 {
            side_b.cross.remove(file);
        }
        // which n lines were the coincidence is not archived: the
        // file's line identities are no longer trustworthy
        side_b.cross_lines.remove(file);
        let side = if added { "added" } else { "removed" };
        applied.push(json!({"file": file, "side": side, "lines": n, "why": why}));
    }
    applied
}

/// The corrections' per-file line counts for one side of one commit.
pub fn per_file_corrections(sha: &str, added: bool) -> PerFile {
    CORRECTIONS
        .iter()
        .filter(|(pre, _, a, _, _)| sha.starts_with(pre) && *a == added)
        .map(|(_, file, _, n, _)| (file.to_string(), *n))
        .collect()
}

pub fn units_for(sha: &str) -> Vec<Value> {
    RELOCATED_UNITS
        .iter()
        .filter(|(pre, _, _)| sha.starts_with(pre))
        .map(|(_, to, units)| {
            json!({"to": to, "units": units.split(',').map(str::trim).collect::<Vec<_>>()})
        })
        .collect()
}
