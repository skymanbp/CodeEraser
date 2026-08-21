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

impl Thresholds {
    /// The ladder must climb, or the warn arm is unreachable. ONE
    /// predicate for the two readers of these keys: scan/wire.rs
    /// refused `fail < warn` and the report.rs mirror judged on
    /// silently, so `ce scan` exited 2 on a ce.toml the MCP scan tool
    /// served a full report from. `fail == 0` is the published "no
    /// hard line" (CE.Scan.Cost.gradeTable), never a low line.
    pub fn ladder_fault(&self) -> Option<String> {
        [
            (
                self.file_lines_warn,
                self.file_lines_fail,
                "file_lines_warn/file_lines_fail",
            ),
            (
                self.fn_lines_warn,
                self.fn_lines_fail,
                "fn_lines_warn/fn_lines_fail",
            ),
        ]
        .into_iter()
        .find(|&(warn, fail, _)| fail != 0 && fail < warn)
        .map(|(warn, fail, keys)| {
            format!(
                "ce.toml [thresholds] {keys}: the fail line {fail} sits below the warn line {warn}"
            )
        })
    }
}

// The guard tier is POLICY, not schema: it validates the declared
// value and renders its own degradation, and three surfaces have to
// agree on both. Re-exported so every existing `config::Guard` /
// `config::PROMOTED_DEFAULT` path keeps working.
mod tier;
pub use tier::{Guard, PROMOTED_DEFAULT, TIERS, tier_of};

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
    /// v0.6 soft-zone curve (plan v2.6 §A): the axis-0 penalty of a
    /// file AT the hard line; absent = the core's default 10.
    pub size_penalty_max: Option<u32>,
    /// v0.6 relative soft line (§B): the multiplicative-MAD exponent
    /// k in S = clamp(median·r^k, [200,500]); absent = default 2.
    pub soft_line_k: Option<u32>,
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

/// Trend-family knobs (M7.5b, trend/1): both OPTIONAL — absent =
/// the core's own defaults (minPoints 3, floor 0 = report-only; the
/// knob rows ride the wire only when declared, the ceilings/27b9bc2
/// pattern). decline_floor_micro is micro-per-mille per day; a
/// declared floor is what arms the fail bit.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TrendCfg {
    pub min_points: Option<u32>,
    pub decline_floor_micro: Option<u64>,
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
    pub trend: TrendCfg,
}

/// Seconds from an env TEST SEAM, else the shipped default — the one
/// shape behind CE_DAEMON_IDLE_SECS (a daemon e2e cannot idle 30 min)
/// and CE_CORE_DEADLINE_SECS (a wedged-core test cannot wait a
/// minute); the ratchet caught the second copy of the parse.
pub fn env_secs(var: &str, default_secs: u64) -> std::time::Duration {
    std::env::var(var)
        .ok()
        .and_then(|s| s.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(std::time::Duration::from_secs(default_secs))
}

impl Config {
    /// Load `ce.toml` for `root`; absent file = defaults.
    ///
    /// The file is looked for at the project ANCHOR above `root`, not
    /// at `root` itself: `ce check cli` used to judge with an empty
    /// config and an empty ratchet — green by having no rules — while
    /// the project's own ce.toml sat one level up. The path the caller
    /// named still bounds what is walked; only the declaration follows
    /// the project (crate::root, and see its note on why the ascent
    /// lives at the state throats rather than inside every family).
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = crate::root::project_root(root).join("ce.toml");
        if !path.is_file() {
            return Ok(Self::default());
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let cfg: Self =
            toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
        // Refused at the LOAD throat, the one both threshold readers
        // pass through — a guard on only the wire path let the
        // auxiliary surfaces judge with a ladder the gate rejects.
        match cfg.thresholds.ladder_fault() {
            Some(fault) => Err(fault),
            None => Ok(cfg),
        }
    }
}
