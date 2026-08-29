//! ce.toml — declarative-only config (thresholds + excludes).
//! Trust model per plan §5.9: no executable fields, ever.

use serde::{Deserialize, Serialize};
use std::path::Path;

// The guard tier is POLICY, not schema: it validates the declared
// value and renders its own degradation, and three surfaces have to
// agree on both. Re-exported so every existing `config::Guard` /
// `config::PROMOTED_DEFAULT` path keeps working.
// The size / complexity ladder, its provenance and its climb rule.
mod thresholds;
pub use thresholds::Thresholds;

mod tier;
pub use tier::{Guard, PROMOTED_DEFAULT, TIERS, tier_of};

// The rulepack (plan v2.13 ①) is its own module for the same reason:
// the class ladder and the fence are policy judged at load.
mod rules;
pub use rules::{CLASS_CAP, ClassCfg, ClassKnobs, RulesCfg};

// The knob fingerprint's canonical form (O39): the effective knob
// set, computed generically over the serialized config.
mod canonical;
pub use canonical::canonical;

/// Dedup ratchet (M2 review R12): `ce dedup --check` fails when the
/// repo's clone-block count exceeds this only-shrink budget.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DedupCfg {
    pub budget: Option<usize>,
}

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
}

impl GraphCfg {
    /// The load-throat refusal: a floor below 1 would call every
    /// node a cycle (the core refuses it too, but a config mistake
    /// must surface with the ce.toml key named, never as a wire
    /// refusal — the ladder_fault stance).
    pub(crate) fn fault(&self) -> Option<String> {
        (self.scc_floor == Some(0)).then(|| {
            "ce.toml [graph] scc_floor must be >= 1 (1 counts self-loops; the shipped floor is 2)"
                .to_string()
        })
    }

    /// The declared roots as the walk spells paths (`rel_str`): `/`
    /// separators, no `./` head, no trailing slash — ONE normalizer
    /// for the two readers (the resolver key and the target set).
    pub(crate) fn declared_roots(&self) -> std::collections::BTreeSet<String> {
        self.crate_roots
            .iter()
            .map(|r| r.replace('\\', "/"))
            .map(|r| r.trim_start_matches("./").trim_end_matches('/').to_string())
            .filter(|r| !r.is_empty())
            .collect()
    }
}

/// Verdict-family knobs (ADR-008 P4): every value is OPTIONAL — an
/// absent key sends no wire row and the core's Cost.hs default
/// judges. Key names mirror the core knob names; the wire codes
/// live in score::knobs as ONE declared table.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct StructureCfg {
    pub layout: std::collections::BTreeMap<String, u32>,
}

/// Trend-family knobs (M7.5b; trend/2 since 2.31.0): both OPTIONAL — absent =
/// the core's own defaults (minPoints 3, floor 0 = report-only; the
/// knob rows ride the wire only when declared, the ceilings/27b9bc2
/// pattern). decline_floor_micro is micro-per-mille per day; a
/// declared floor is what arms the fail bit.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct TrendCfg {
    pub min_points: Option<u32>,
    pub decline_floor_micro: Option<u64>,
}

impl TrendCfg {
    /// The core's own defaults as declared values (`CE.Trend.Cost`
    /// minPoints 3, floor 0): the digest's effective default, pinned
    /// live by core_wire's mirror gate against the trend echo.
    pub(crate) fn core() -> Self {
        Self {
            min_points: Some(3),
            decline_floor_micro: Some(0),
        }
    }
}

/// deny_unknown_fields everywhere (ADR-008 P4): a mistyped policy
/// key used to be SILENTLY dropped — a config that looks live and
/// does nothing is the exact failure mode this repo exists to fight.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub thresholds: Thresholds,
    /// Extra exclude globs, added on top of built-in defaults (§4.1).
    pub exclude: Vec<String>,
    pub guard: Guard,
    pub dedup: DedupCfg,
    pub(crate) graph: GraphCfg,
    pub score: ScoreCfg,
    pub(crate) structure: StructureCfg,
    pub(crate) trend: TrendCfg,
    /// Path classes with their own size/complexity knobs (plan v2.13
    /// ①, `[[rules.class]]`); absent = one global table, wire unchanged.
    pub rules: RulesCfg,
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
    /// The KNOB FINGERPRINT (5.1.0 as knobsDigest, widened at 6.0.0,
    /// canonical since O39), or None when this repo judges as the
    /// shipped default does — such a repo keeps the bytes it had
    /// before the fence existed.
    ///
    /// The whole parsed config, not a chosen table. The first version
    /// fingerprinted `[[rules.class]]` alone, and an adversarial
    /// review found the scope wrong within the hour: two lines of
    /// `[score]` — `viol_cost = 0` — pin the score at the scale, so
    /// `ce check --fail-under 946` can never fail again, and
    /// `tol_abs = 100000` erases the ratchet's own tolerance. Both
    /// move the same gates a glob edit moves, neither touched the
    /// class table, and neither asked anyone to name a floor. Picking
    /// which tables to fence is how that happens; fingerprinting the
    /// config is how it stops happening, including for the knob
    /// nobody has added yet.
    ///
    /// The hashed bytes are the CANONICAL tree (config/canonical.rs):
    /// the knobs whose effective value differs from the shipped
    /// default, as key-ordered JSON — so comments, key order, a knob
    /// spelled at its default and an optional knob nobody declared
    /// all leave it alone, and no value can be read as structure. The
    /// literal a fixed ce.toml hashes to is a compatibility surface
    /// (it sits in downstream baselines), frozen in
    /// config_contract::the_digest_of_a_fixed_declaration_is_frozen.
    pub fn knobs_digest(&self) -> Option<u64> {
        let tree = canonical(self);
        let bytes = serde_json::to_vec(&tree).ok()?;
        (tree != serde_json::Value::Object(Default::default()))
            .then(|| crate::score::baseline::fnv1a(&[&bytes]))
    }

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
        match cfg
            .thresholds
            .ladder_fault()
            .or_else(|| cfg.rules.fault(&cfg.thresholds))
            .or_else(|| cfg.graph.fault())
            .or_else(|| cfg.globs_fault(path.parent().unwrap_or(root)))
        {
            Some(fault) => Err(fault),
            None => Ok(cfg),
        }
    }

    /// Every ce.toml glob — the exclude list, each class's globs,
    /// `[graph] entry_globs` — compiles at the load throat in the one
    /// dialect (scan::globs), so a pattern the dialect would silently
    /// drop or misread (a `#` comment, an escaped `\\`, a `!`) is a
    /// named error with the fix in it before any reader judges.
    fn globs_fault(&self, root: &Path) -> Option<String> {
        use crate::scan::globs;
        let mut b = ignore::overrides::OverrideBuilder::new(root);
        let exclude = self
            .exclude
            .iter()
            .find_map(|g| globs::add_user_glob(&mut b, g, true, "exclude").err());
        let classes = || {
            self.rules.class.iter().find_map(|c| {
                let what = format!("[[rules.class]] {:?}", c.name);
                globs::compile_inclusions(root, &c.globs, &what).err()
            })
        };
        let entries = || {
            globs::compile_inclusions(root, &self.graph.entry_globs, "[graph] entry_globs").err()
        };
        exclude.or_else(classes).or_else(entries)
    }
}

#[cfg(test)]
#[path = "../tests/unit/config_knob_fingerprint.rs"]
mod knob_fingerprint;
