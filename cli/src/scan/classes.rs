//! The rulepack's MATCHING half (plan v2.13 ①): every measured path
//! tries the declared classes in array order and the first match owns
//! it — 1-based, 0 = the default class. Globs compile through the
//! SAME override compiler the exclude list uses (walk::add_user_glob:
//! one dialect, the third-dialect refusal) as INCLUSION patterns.
//! Paths and class names stay on this side of the wire (§5.9.2); the
//! index is what rides.

use crate::config::RulesCfg;
use ignore::overrides::{Override, OverrideBuilder};
use std::path::Path;

pub struct Classes {
    matchers: Vec<Override>,
}

impl Classes {
    /// One override set per class, compiled once per run — the same
    /// throughput class as the exclude set every walk already pays.
    pub fn compile(root: &Path, rules: &RulesCfg) -> Result<Self, String> {
        let mut matchers = Vec::with_capacity(rules.class.len());
        for c in &rules.class {
            let what = format!("[[rules.class]] {:?}", c.name);
            let mut b = OverrideBuilder::new(root);
            for glob in &c.globs {
                crate::scan::walk::add_user_glob(&mut b, glob, false, &what)?;
            }
            matchers.push(b.build().map_err(|e| format!("ce.toml {what}: {e}"))?);
        }
        Ok(Classes { matchers })
    }

    /// Any class declared at all — the continuous rows carry their
    /// class column exactly then (a legacy request otherwise, so an
    /// undeclared repo's wire bytes never move).
    pub fn declared(&self) -> bool {
        !self.matchers.is_empty()
    }

    /// The 1-based index of the first class matching `rel` (the
    /// '/'-spelled repo-relative path every report keys on), 0 when
    /// none does. A zero-hit class is legal — a clean tree may hold
    /// no generated file today — and never an error here.
    pub fn class_of(&self, rel: &str) -> u64 {
        let path = Path::new(rel);
        self.matchers
            .iter()
            .position(|m| matches!(m.matched(path, false), ignore::Match::Whitelist(_)))
            .map_or(0, |i| i as u64 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClassCfg;

    type Decl<'a> = (&'a str, &'a [&'a str]);

    fn rules(classes: &[Decl]) -> RulesCfg {
        RulesCfg {
            class: classes
                .iter()
                .map(|(name, globs)| ClassCfg {
                    name: name.to_string(),
                    globs: globs.iter().map(|g| g.to_string()).collect(),
                    knobs: Default::default(),
                })
                .collect(),
        }
    }

    /// Declaration order is the tiebreak (C3): the same overlapping
    /// pair flipped hands the shared path to the other class, an
    /// unmatched path is class 0, and a Windows-spelled glob matches
    /// through the same normalization the exclude list gets.
    #[test]
    fn first_declared_match_owns_the_path() {
        let root = Path::new(".");
        let owner = |classes: &[Decl], path: &str| {
            Classes::compile(root, &rules(classes))
                .expect("compile")
                .class_of(path)
        };
        let (tests, cli): (Decl, Decl) = (("tests", &["cli/tests/**"]), ("cli", &["cli/**"]));
        assert_eq!(owner(&[tests, cli], "cli/tests/x.rs"), 1);
        assert_eq!(owner(&[tests, cli], "cli/src/x.rs"), 2);
        assert_eq!(owner(&[tests, cli], "core/app/X.hs"), 0);
        assert_eq!(
            owner(&[cli, tests], "cli/tests/x.rs"),
            1,
            "flipped order, flipped owner"
        );
        assert_eq!(
            owner(
                &[("gen", &["src\\generated\\*.rs"])],
                "src/generated/api.rs"
            ),
            1
        );
        assert!(
            !Classes::compile(root, &RulesCfg::default())
                .expect("empty")
                .declared()
        );
        let err = Classes::compile(root, &rules(&[("neg", &["!src/**"])]))
            .err()
            .expect("'!' refuses");
        assert!(err.contains("without '!'"), "{err}");
    }
}
