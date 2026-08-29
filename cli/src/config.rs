//! ce.toml — declarative-only config (thresholds + excludes).
//! Trust model per plan §5.9: no executable fields, ever.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Thresholds; defaults from DEVELOPMENT_PLAN.md §4.1 (provenance:
/// ESLint max-lines=300, Sonar S104=750/S138=75, ESLint fn=50,
/// Pylint max-args=5, Sonar S3776 CoC=15, lizard CC=15).
#[derive(Debug, Clone, Deserialize, Serialize)]
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

// The rulepack (plan v2.13 ①) is its own module for the same reason:
// the class ladder and the fence are policy judged at load.
mod rules;
pub use rules::{CLASS_CAP, ClassCfg, ClassKnobs, RulesCfg};

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
}

impl GraphCfg {
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
    /// The KNOB FINGERPRINT (5.1.0 as knobsDigest, widened at 6.0.0),
    /// or None when this repo declares nothing — a repo whose config
    /// is the shipped default must keep the bytes it had before the
    /// fence existed.
    ///
    /// The whole parsed config, not a chosen table. The first version
    /// fingerprinted `[[rules.class]]` alone, and an adversarial
    /// review found the scope wrong within the hour: two lines of
    /// `[score]` — `viol_cost = 0` — pin the score at the scale, so
    /// `ce check --fail-under 940` can never fail again, and
    /// `tol_abs = 100000` erases the ratchet's own tolerance. Both
    /// move the same gates a glob edit moves, neither touched the
    /// class table, and neither asked anyone to name a floor. Picking
    /// which tables to fence is how that happens; fingerprinting the
    /// config is how it stops happening, including for the knob
    /// nobody has added yet.
    ///
    /// Serialized JSON is the canonical form: it is deterministic
    /// (struct field order), it escapes its own delimiters so no
    /// value can be read as structure, and it covers a new field the
    /// day the field is declared rather than the day someone
    /// remembers to add it here. Comments and key order in ce.toml do
    /// not move it, because it fingerprints the PARSE, not the file.
    pub fn knobs_digest(&self) -> Option<u64> {
        let declared = serde_json::to_vec(self).ok()?;
        let shipped = serde_json::to_vec(&Config::default()).ok()?;
        (declared != shipped).then(|| crate::score::baseline::fnv1a(&[&declared]))
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
        {
            Some(fault) => Err(fault),
            None => Ok(cfg),
        }
    }
}

#[cfg(test)]
mod knob_fingerprint {
    use super::*;

    fn with_class(name: &str, globs: &[&str], tol: Option<usize>) -> Config {
        Config {
            rules: RulesCfg {
                class: vec![ClassCfg {
                    name: name.into(),
                    globs: globs.iter().map(|g| (*g).to_string()).collect(),
                    knobs: ClassKnobs {
                        ratchet_tolerance: tol,
                        ..ClassKnobs::default()
                    },
                }],
            },
            ..Config::default()
        }
    }

    /// A repo that declares nothing has no fingerprint. Absence is the
    /// state the fence must leave untouched, byte for byte, and it is
    /// what makes the fence free for everyone who never opted in.
    #[test]
    fn the_shipped_default_has_no_fingerprint() {
        assert_eq!(Config::default().knobs_digest(), None);
    }

    /// The two knobs the adversarial review turned into a bypass. This
    /// is the reason the fingerprint covers the whole config instead of
    /// the class table: `viol_cost = 0` pins the score at the scale so
    /// `--fail-under` can never fail, and `tol_abs` erases the
    /// ratchet's tolerance. Neither touches [[rules.class]].
    #[test]
    fn the_score_knobs_that_bypassed_the_gates_move_it() {
        let base = Config::default().knobs_digest();
        let viol = Config {
            score: ScoreCfg {
                viol_cost: Some(0),
                ..ScoreCfg::default()
            },
            ..Config::default()
        };
        let tol = Config {
            score: ScoreCfg {
                tol_abs: Some(100_000),
                ..ScoreCfg::default()
            },
            ..Config::default()
        };
        assert_ne!(base, viol.knobs_digest(), "viol_cost");
        assert_ne!(base, tol.knobs_digest(), "tol_abs");
        assert_ne!(
            viol.knobs_digest(),
            tol.knobs_digest(),
            "and from each other"
        );
    }

    /// Everything a rulepack declaration can say still moves it —
    /// including declaration ORDER, which is precedence, and a knob set
    /// to zero, which is a claim and not an absence.
    #[test]
    fn the_rulepack_still_moves_it_in_every_part() {
        let a = with_class("vendored", &["vendor/**"], None).knobs_digest();
        assert!(a.is_some());
        assert_ne!(a, with_class("vendor", &["vendor/**"], None).knobs_digest());
        assert_ne!(
            a,
            with_class("vendored", &["vendor/*"], None).knobs_digest()
        );
        assert_ne!(
            a,
            with_class("vendored", &["vendor/**"], Some(0)).knobs_digest()
        );
        let two = |x: &str, y: &str| Config {
            rules: RulesCfg {
                class: [x, y]
                    .iter()
                    .map(|n| ClassCfg {
                        name: (*n).into(),
                        globs: vec![format!("{n}/**")],
                        knobs: ClassKnobs::default(),
                    })
                    .collect(),
            },
            ..Config::default()
        };
        assert_ne!(
            two("a", "b").knobs_digest(),
            two("b", "a").knobs_digest(),
            "declaration order IS precedence"
        );
    }

    /// An exclude glob drops files from the walk, and with them their
    /// ratchet rows — the third road the review found, fenced by the
    /// same scalar as the first two.
    #[test]
    fn an_exclude_glob_moves_it() {
        let excluded = Config {
            exclude: vec!["vendor/**".into()],
            ..Config::default()
        };
        assert_ne!(Config::default().knobs_digest(), excluded.knobs_digest());
    }

    /// No value can be read as structure: a name carrying the JSON the
    /// encoding uses is escaped, not spliced. The class-only draft
    /// separated fields with a NUL and its own test found the collision
    /// immediately; serialized JSON has no such seam to exploit.
    #[test]
    fn a_name_cannot_impersonate_the_encoding() {
        assert_ne!(
            with_class("a", &["b"], None).knobs_digest(),
            with_class("a\",\"globs\":[\"b", &[], None).knobs_digest(),
        );
    }
}
