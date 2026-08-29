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

    /// The threshold table `rel` is measured against locally (P3,
    /// 3.2.0): its class's effective lines, or the global table for
    /// class 0 — the same reading the core takes from gradeOverrides,
    /// so the scan's pinned mirror keeps proving the wire equal.
    pub fn thresholds_for(
        &self,
        cfg: &crate::config::Config,
        rel: &str,
    ) -> crate::config::Thresholds {
        match self.class_of(rel) {
            0 => cfg.thresholds.clone(),
            c => cfg.rules.class[c as usize - 1].effective(&cfg.thresholds),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/scan/classes.rs"]
mod tests;
