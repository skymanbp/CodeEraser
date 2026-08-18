//! ce.toml — declarative-only config (thresholds + excludes).
//! Trust model per plan §5.9: no executable fields, ever.

use serde::Deserialize;
use std::path::Path;

/// Thresholds; defaults from DEVELOPMENT_PLAN.md §4.1 (provenance:
/// ESLint max-lines=300, Sonar S104=750/S138=75, ESLint fn=50,
/// Pylint max-args=5, Sonar S3776 CoC=15, lizard CC=15).
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
pub struct Guard {
    /// Explicit global tier: "observe" | "warn" | "ask" | "deny".
    /// Unset = the §4.2 route defaults, resolved per rule class by
    /// `tier` (step 2 landed 2026-08-11 — CHANGELOG).
    pub mode: Option<String>,
}

/// §4.2 step-3 route default for the two promoted PreToolUse classes
/// (T1/T2 duplicate write, hard-budget breach): deny at 1.0, decided
/// M7-P2 on the recorded FPR ledger (CHANGELOG 2026-08-17). ONE
/// constant — guard.rs and health.rs both read it, so the enforced
/// tier and the reported tier cannot drift apart.
pub const PROMOTED_DEFAULT: &str = "deny";

impl Guard {
    /// Effective tier for one rule class: an explicit `[guard] mode`
    /// overrides every class; otherwise the plan-§4.2 route default
    /// for that class applies (PROMOTED_DEFAULT for the two classes
    /// promoted through the FPR gates, "observe" for everything else).
    pub fn tier(&self, route_default: &str) -> String {
        self.mode
            .clone()
            .unwrap_or_else(|| route_default.to_string())
    }
}

/// Dedup ratchet (M2 review R12): `ce dedup --check` fails when the
/// repo's clone-block count exceeds this only-shrink budget.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DedupCfg {
    pub budget: Option<usize>,
}

/// Graph/deadcode settings (M5-2h). entry_globs marks extra
/// liveness roots beyond the mechanical conventions (main-shaped
/// files, test conventions, doc entries) — flag bit 3 on the wire.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GraphCfg {
    pub entry_globs: Vec<String>,
}

/// Verdict-family knobs (ADR-008 P4): every value is OPTIONAL — an
/// absent key sends no wire row and the core's Cost.hs default
/// judges. Key names mirror the core knob names; the wire codes
/// live in score::knobs as ONE declared table.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScoreCfg {
    /// Per-axis weight numerators by axis NAME (size / complexity /
    /// clone / docdup / deadcode / churn / cycle); unlisted axes
    /// keep the decision-⑦ equal weight.
    pub weights: std::collections::BTreeMap<String, u32>,
    pub dead_indeg_ceil: Option<u32>,
    pub rewrite_num: Option<u32>,
    pub rewrite_den: Option<u32>,
    pub cochange_floor: Option<u32>,
    pub viol_cost: Option<u32>,
    pub default_weight: Option<u32>,
    pub score_scale: Option<u32>,
    pub tol_num: Option<u32>,
    pub tol_den: Option<u32>,
    pub tol_abs: Option<u32>,
}

/// Structure declaration layer (M6 S3a, design booklet §2 row A):
/// the OPTIONAL layout template the χ² divergence judges against.
/// Absent = the self-referential floor alone (row C). Keys are
/// directory paths relative to the root ("." = the root itself, the
/// catch-all bin under deepest-owner semantics); values are
/// relative weights >= 1. deny_unknown_fields from day one — the
/// review C2 lesson, not a later retrofit.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StructureCfg {
    pub layout: std::collections::BTreeMap<String, u32>,
}

/// deny_unknown_fields everywhere (ADR-008 P4): a mistyped policy
/// key used to be SILENTLY dropped — a config that looks live and
/// does nothing is the exact failure mode this repo exists to fight.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub thresholds: Thresholds,
    /// Extra exclude globs, added on top of built-in defaults (§4.1).
    pub exclude: Vec<String>,
    pub guard: Guard,
    pub dedup: DedupCfg,
    pub graph: GraphCfg,
    pub score: ScoreCfg,
    pub structure: StructureCfg,
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
