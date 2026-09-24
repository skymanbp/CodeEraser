//! `[graph]` — the resolver's declarations (M5-2h entry globs, plan
//! v2.18 step #12 crate roots, 6.4.0 cycle floor, plan v2.30 search
//! roots), judged at load like the rulepack. Split from config.rs
//! when the search-root table pushed that file past the 300-line line.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Graph/deadcode settings (M5-2h). entry_globs marks extra
/// liveness roots beyond the mechanical conventions (main-shaped
/// files, test conventions, doc entries) — flag bit 3 on the wire.
/// crate_roots (plan v2.18 step #12) names the Rust crate roots of a
/// tree whose manifest lives ELSEWHERE — the test-suite submodule is
/// a slice of the `cli` package, and its `it/main.rs` is a cargo test
/// target only in the superproject's Cargo.toml. A declared root is
/// everything a manifest target is: the Rust ladder mounts its `mod`
/// children in its own directory and anchors `crate::` paths there
/// (ladder/rs.rs), and it is a declared build target for the entry
/// role (deadcode/targets.rs, role 6). Root-relative exact paths,
/// `/`-spelled; a tree with a manifest needs none.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct GraphCfg {
    pub entry_globs: Vec<String>,
    pub crate_roots: Vec<String>,
    /// The cycle floor (6.4.0, O59): the smallest strongly connected
    /// component that counts as a cycle — the ONE knob the graph's
    /// cycle table (`sccFloor`) and the verdict's cycle axis
    /// (`cycleFloor`, threshold code 7) both read, so a singleton SCC
    /// is a cycle on both faces or on neither. Absent = the shipped 2
    /// (a lone node is never a cycle); 1 counts a file exactly when
    /// it carries a self-arc, and the verdict then needs the
    /// self-loop table the graph reply projects. 0 is refused at load.
    pub scc_floor: Option<u32>,
    /// `[graph.search_roots]` (plan v2.30): language name → the
    /// directories that language's ladder searches after the site's
    /// own rung and before its build configuration — C / C++ include
    /// roots under the shared key `c`, and the Lua, Java, R and HTML
    /// roots as their steps land. One table, one key form, for what is
    /// one concept five times (booklet §14 ruling 8).
    /// Root-relative directories; a key naming no such language is
    /// refused at load, and a directory holding no walked file is
    /// refused by the walk (the crate_roots posture).
    pub search_roots: BTreeMap<String, Vec<String>>,
}

/// The languages whose ladders read a declared search root.
const SEARCH_ROOT_LANGS: [&str; 5] = ["c", "lua", "java", "r", "html"];

impl GraphCfg {
    /// The load-throat refusal: a floor below 1 would call every
    /// node a cycle (the core refuses it too, but a config mistake
    /// must surface with the ce.toml key named, never as a wire
    /// refusal — the ladder_fault stance); a search-root key that is
    /// no language name is a typo that would otherwise sit in the
    /// file doing nothing.
    pub(crate) fn fault(&self) -> Option<String> {
        if self.scc_floor == Some(0) {
            return Some(
                "ce.toml [graph] scc_floor must be >= 1 (1 counts self-loops; the shipped floor is 2)"
                    .to_string(),
            );
        }
        let stray = self
            .search_roots
            .keys()
            .find(|k| !SEARCH_ROOT_LANGS.contains(&k.as_str()))?;
        Some(format!(
            "ce.toml [graph.search_roots] names {stray:?}; the languages with search roots are {}",
            SEARCH_ROOT_LANGS.join(" ")
        ))
    }

    /// The declared roots as the walk spells paths (`rel_str`): `/`
    /// separators, no `./` head, no trailing slash — ONE normalizer
    /// for the two readers (the resolver key and the target set).
    pub(crate) fn declared_roots(&self) -> BTreeSet<String> {
        self.crate_roots
            .iter()
            .map(|r| normalized(r))
            .filter(|r| !r.is_empty())
            .collect()
    }

    /// The declared search roots per language, spelled the same way;
    /// `.` (and an empty entry) is the repo root itself.
    pub(crate) fn declared_search_roots(&self) -> BTreeMap<String, BTreeSet<String>> {
        self.search_roots
            .iter()
            .map(|(lang, dirs)| (lang.clone(), dirs.iter().map(|d| normalized(d)).collect()))
            .collect()
    }
}

/// A declared path as `walk::rel_str` spells one: `/` separators, no
/// `./` head, no trailing slash, and `.` is the root (empty).
fn normalized(declared: &str) -> String {
    let path = declared.replace('\\', "/");
    let path = path.trim_start_matches("./").trim_end_matches('/');
    if path == "." {
        String::new()
    } else {
        path.to_string()
    }
}
