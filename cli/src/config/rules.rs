//! Rulepack v1 (plan v2.13 ①): path classes with their own size and
//! complexity knobs — the DECLARATION half. Names and globs are local
//! facts (matching and rendering, never the wire — §5.9.2); what
//! crosses is a class's 1-based declaration index on each continuous
//! row and its knob rows (score::knobs::class_knob_rows). Matching
//! lives in scan::classes, through the same override compiler the
//! exclude list uses — one glob dialect, by decision.

use super::Thresholds;
use serde::{Deserialize, Serialize};

/// The allocation fence: a 65th class refuses at load. A fence, not a
/// quota (the softKMax stance) — every class still argues its
/// judgment value at review, ADR-008's "no code for a share" line
/// extended to configuration.
pub const CLASS_CAP: usize = 64;

/// Array order IS precedence: the first class whose globs match a
/// path owns it (the verdictTable first-match reading), and a path no
/// class matches is class 0 — the global table.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesCfg {
    pub class: Vec<ClassCfg>,
}

/// name and globs are REQUIRED keys — serde names the missing one;
/// only the knobs default (absent = inherit every global line).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClassCfg {
    pub name: String,
    pub globs: Vec<String>,
    #[serde(default)]
    pub knobs: ClassKnobs,
}

/// Absent = inherit the global [thresholds] value. The score reads
/// the size and complexity ceilings as the ceilings table's own codes
/// 0 / 1 / 2 (sizeCeil / cocCeil / sizeHard) with a class dimension —
/// no new code was minted; the scan ladder (P3, 3.2.0) reads the six
/// line knobs as per-class grade overrides on its codes 0 / 1 / 4.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClassKnobs {
    pub file_lines_warn: Knob,
    pub file_lines_fail: Knob,
    pub cognitive_warn: Knob,
    /// The class's own complexity wall (plan v2.24). Absent inherits
    /// the global `cognitive_fail`, which ships 0 — so a class opts
    /// in exactly like the global table does, and a class that says
    /// nothing keeps sending no code-4 fail line at all.
    pub cognitive_fail: Knob,
    pub fn_lines_warn: Knob,
    pub fn_lines_fail: Knob,
    /// The class's OWN ratchet allowance in lines (5.1.0, plan v2.14
    /// ②): declared, it replaces both global legs — a class with 0
    /// may not grow by a single line, and the global max(+2%, +10)
    /// does not rescue it because it is never consulted. Absent =
    /// the global legs, exactly as before. Absolute rather than
    /// proportional on purpose: the trees that want this knob
    /// (vendored, fixtures) want zero or a fixed slack, and a
    /// percentage of a large file is the unearned growth this knob
    /// exists to take away.
    pub ratchet_tolerance: Knob,
    /// The cognitive-complexity sibling (6.4.0, O37): declared, it
    /// replaces `ratchet_tolerance` for the fn-CoC rows ALONE — a
    /// class may freeze its lines and still allow CoC growth, or the
    /// reverse. Absent = `ratchet_tolerance` (then the global legs)
    /// judges both metrics exactly as before; the wire carries it as
    /// class knob code 4, so an undeclared repo sends no byte.
    pub cognitive_ratchet_tolerance: Knob,
}

/// One class knob: a declared value, or absent — inherit. Named
/// because every field of the table is one, and the table is read
/// as a table (knobs.rs, wire.rs), never field by field.
pub type Knob = Option<usize>;

impl ClassCfg {
    /// The class's effective table: its overrides over the global one.
    pub fn effective(&self, global: &Thresholds) -> Thresholds {
        let k = &self.knobs;
        Thresholds {
            file_lines_warn: k.file_lines_warn.unwrap_or(global.file_lines_warn),
            file_lines_fail: k.file_lines_fail.unwrap_or(global.file_lines_fail),
            cognitive_warn: k.cognitive_warn.unwrap_or(global.cognitive_warn),
            cognitive_fail: k.cognitive_fail.unwrap_or(global.cognitive_fail),
            fn_lines_warn: k.fn_lines_warn.unwrap_or(global.fn_lines_warn),
            fn_lines_fail: k.fn_lines_fail.unwrap_or(global.fn_lines_fail),
            ..global.clone()
        }
    }
}
impl RulesCfg {
    /// The load-throat refusals, in declaration order: the fence, an
    /// empty or twice-declared name, a class with nothing to match (a
    /// declaration that can never do anything is the exact silent
    /// failure this file exists to refuse), and the per-class ladder —
    /// judged on the EFFECTIVE lines through the one predicate the
    /// global [thresholds] already answers to.
    pub fn fault(&self, global: &Thresholds) -> Option<String> {
        if self.class.len() > CLASS_CAP {
            return Some(format!(
                "ce.toml [[rules.class]]: {} classes declared, the fence is {CLASS_CAP}",
                self.class.len()
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.class {
            let tag = format!("ce.toml [[rules.class]] {:?}", c.name);
            if c.name.is_empty() {
                return Some(format!("{tag}: a class needs a name"));
            }
            if !seen.insert(c.name.as_str()) {
                return Some(format!("{tag}: declared twice"));
            }
            if c.globs.is_empty() {
                return Some(format!(
                    "{tag}: no globs — nothing could ever be in this class"
                ));
            }
            if let Some(f) = c.effective(global).ladder_fault() {
                return Some(format!(
                    "{tag}: {}",
                    f.trim_start_matches("ce.toml [thresholds] ")
                ));
            }
        }
        None
    }
}
