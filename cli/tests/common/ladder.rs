//! Ladder-test arrange + act helpers (the hooks.rs precedent:
//! common/ is one module per concern, each binary uses a subset —
//! the allows in mod.rs are module-level and cover this file too).

use codeeraser::graph::ladder::{self, Outcome, Reason, Scope};
use codeeraser::scan::lang::Lang;
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
    pub memo: ladder::Memo,
}

pub fn fixture(tag: &str, tree: &[(&str, &str)]) -> Fixture {
    let dir = super::tmp(tag);
    let (files, configs) = materialize(&dir, tree);
    Fixture {
        dir,
        files,
        configs,
        memo: Default::default(),
    }
}

impl Fixture {
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            files: &self.files,
            configs: &self.configs,
            root: &self.dir,
            memo: &self.memo,
        }
    }
}

/// (lang, site kind, from-file, spec) → expected outcome. Aliases
/// keep every row on one line: the table IS the spec, scannable or
/// dead.
pub type Case = (Lang, &'static str, &'static str, &'static str, Outcome);

/// Drive a case table through the dispatcher against one
/// materialized fixture — the shared act + assert throat. Case rows
/// carry no line: table fixtures hold no inline modules, so line 1
/// is exact (the inline-module cases pass real lines directly).
pub fn run_cases(fx: &Fixture, cases: Vec<Case>) {
    let scope = fx.scope();
    for (lang, kind, from, spec, want) in cases {
        let got = ladder::resolve(lang, &site(kind, from, spec, 1), &scope);
        assert_eq!(got, want, "{lang:?} {kind} {spec:?}");
    }
}

/// A one-off site for direct dispatch — the inline-module cases need
/// a REAL line; the tables go through run_cases at line 1.
pub fn site(
    kind: &'static str,
    from: &'static str,
    spec: &'static str,
    line: usize,
) -> ladder::Site<'static> {
    ladder::Site {
        kind,
        from,
        spec,
        line,
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

/// Resolved through a re-export surface (§4 R5 amendment).
pub fn via(path: &str, rung: u8) -> Outcome {
    Outcome::ResolvedVia {
        path: path.to_string(),
        rung,
    }
}

pub fn ext(rung: u8) -> Outcome {
    Outcome::External { rung }
}

pub fn sec(path: &str, slug: &str, rung: u8) -> Outcome {
    Outcome::ResolvedSection {
        path: path.to_string(),
        slug: Some(slug.to_string()),
        rung,
    }
}

/// The anchored link whose section claim degraded to a file-level
/// edge (design: ambiguous_anchor).
pub fn secf(path: &str, rung: u8) -> Outcome {
    Outcome::ResolvedSection {
        path: path.to_string(),
        slug: None,
        rung,
    }
}
