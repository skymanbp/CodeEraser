//! Ladder-test arrange helpers (the hooks.rs precedent: common/ is
//! one module per concern, each binary uses a subset — the allows in
//! mod.rs are module-level and cover this file too).

use codeeraser::graph::ladder::{Outcome, Reason, Scope};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Materialize a ladder fixture tree and collect what the real walk
/// would hand the resolver: the in-scope lang files (node_modules is
/// never entered) plus the resolver-config paths.
pub fn materialize(dir: &Path, tree: &[(&str, &str)]) -> (BTreeSet<String>, Vec<String>) {
    let mut files = BTreeSet::new();
    let mut configs = Vec::new();
    for (rel, content) in tree {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, content).expect(rel);
        if rel.starts_with("node_modules/") {
            continue;
        }
        if codeeraser::scan::lang::Lang::from_path(&path).is_some() {
            files.insert(rel.to_string());
        }
        if codeeraser::graph::store::is_resolver_config(&path) {
            configs.push(rel.to_string());
        }
    }
    (files, configs)
}

/// A materialized ladder fixture that OWNS what a Scope borrows —
/// the shared arrange throat of every ladder test (the ratchet
/// caught three copies of the materialize + Scope stanza).
pub struct Fixture {
    pub dir: PathBuf,
    pub files: BTreeSet<String>,
    pub configs: Vec<String>,
}

pub fn fixture(tag: &str, tree: &[(&str, &str)]) -> Fixture {
    let dir = super::tmp(tag);
    let (files, configs) = materialize(&dir, tree);
    Fixture {
        dir,
        files,
        configs,
    }
}

impl Fixture {
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            files: &self.files,
            configs: &self.configs,
            root: &self.dir,
        }
    }
}

/// Ladder case-table constructors: resolved file, resolved package
/// directory (Go granularity), refusal, external.
pub fn ok(path: &str, rung: u8) -> Outcome {
    Outcome::Resolved {
        path: path.to_string(),
        rung,
    }
}

pub fn pkg(dir: &str, rung: u8) -> Outcome {
    Outcome::ResolvedPackage {
        dir: dir.to_string(),
        rung,
    }
}

pub fn no(reason: Reason) -> Outcome {
    Outcome::Unresolved(reason)
}

pub fn ext(rung: u8) -> Outcome {
    Outcome::External { rung }
}
