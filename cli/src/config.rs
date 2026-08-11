//! ce.toml — declarative-only config (thresholds + excludes).
//! Trust model per plan §5.9: no executable fields, ever.

use serde::Deserialize;
use std::path::Path;

/// Thresholds; defaults from DEVELOPMENT_PLAN.md §4.1 (provenance:
/// ESLint max-lines=300, Sonar S104=750/S138=75, ESLint fn=50,
/// Pylint max-args=5, Sonar S3776 CoC=15, lizard CC=15).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Thresholds {
    pub file_lines_warn: usize,
    pub file_lines_fail: usize,
    pub fn_lines_warn: usize,
    pub fn_lines_fail: usize,
    pub params_warn: usize,
    pub cyclomatic_warn: usize,
    pub cognitive_warn: usize,
    pub nesting_warn: usize,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            file_lines_warn: 300,
            file_lines_fail: 750,
            fn_lines_warn: 50,
            fn_lines_fail: 75,
            params_warn: 5,
            cyclomatic_warn: 15,
            cognitive_warn: 15,
            nesting_warn: 4,
        }
    }
}

/// Passive-guard settings (plan §4.2, decision D-4 gradual rollout:
/// observe → warn → ask → deny, promotion gated on measured FPR).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Guard {
    /// Explicit global tier: "observe" | "warn" | "ask" | "deny".
    /// Unset = the §4.2 route defaults, resolved per rule class by
    /// `tier` (step 2 landed 2026-08-11 — CHANGELOG).
    pub mode: Option<String>,
}

impl Guard {
    /// Effective tier for one rule class: an explicit `[guard] mode`
    /// overrides every class; otherwise the plan-§4.2 route default
    /// for that class applies ("ask" for the two classes promoted
    /// after the M4 FPR gate, "observe" for everything else).
    pub fn tier(&self, route_default: &str) -> String {
        self.mode
            .clone()
            .unwrap_or_else(|| route_default.to_string())
    }
}

/// Dedup ratchet (M2 review R12): `ce dedup --check` fails when the
/// repo's clone-block count exceeds this only-shrink budget.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DedupCfg {
    pub budget: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub thresholds: Thresholds,
    /// Extra exclude globs, added on top of built-in defaults (§4.1).
    pub exclude: Vec<String>,
    pub guard: Guard,
    pub dedup: DedupCfg,
}

impl Config {
    /// Load `ce.toml` from `root`; absent file = defaults.
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join("ce.toml");
        if !path.is_file() {
            return Ok(Self::default());
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
    }
}
