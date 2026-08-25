//! Rulepack v1 (plan v2.13 ①): path classes with their own size and
//! complexity knobs — the DECLARATION half. Names and globs are local
//! facts (matching and rendering, never the wire — §5.9.2); what
//! crosses is a class's 1-based declaration index on each continuous
//! row and its knob rows (score::knobs::class_knob_rows). Matching
//! lives in scan::classes, through the same override compiler the
//! exclude list uses — one glob dialect, by decision.

use super::Thresholds;
use serde::Deserialize;

/// The allocation fence: a 65th class refuses at load. A fence, not a
/// quota (the softKMax stance) — every class still argues its
/// judgment value at review, ADR-008's "no code for a share" line
/// extended to configuration.
pub const CLASS_CAP: usize = 64;

/// Array order IS precedence: the first class whose globs match a
/// path owns it (the verdictTable first-match reading), and a path no
/// class matches is class 0 — the global table.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesCfg {
    pub class: Vec<ClassCfg>,
}

/// name and globs are REQUIRED keys — serde names the missing one;
/// only the knobs default (absent = inherit every global line).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassCfg {
    pub name: String,
    pub globs: Vec<String>,
    #[serde(default)]
    pub knobs: ClassKnobs,
}

/// Absent = inherit the global [thresholds] value. The score reads
/// the first three as the ceilings table's own codes 0 / 1 / 2
/// (sizeCeil / cocCeil / sizeHard) with a class dimension — no new
/// code was minted; the scan ladder (P3, 3.2.0) reads all five as
/// per-class grade overrides on its codes 0 / 1 / 4.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClassKnobs {
    pub file_lines_warn: Option<usize>,
    pub file_lines_fail: Option<usize>,
    pub cognitive_warn: Option<usize>,
    pub fn_lines_warn: Option<usize>,
    pub fn_lines_fail: Option<usize>,
    /// The class's OWN ratchet allowance in lines (5.1.0, plan v2.14
    /// ②): declared, it replaces both global legs — a class with 0
    /// may not grow by a single line, and the global max(+2%, +10)
    /// does not rescue it because it is never consulted. Absent =
    /// the global legs, exactly as before. Absolute rather than
    /// proportional on purpose: the trees that want this knob
    /// (vendored, fixtures) want zero or a fixed slack, and a
    /// percentage of a large file is the unearned growth this knob
    /// exists to take away.
    pub ratchet_tolerance: Option<usize>,
}

impl ClassKnobs {
    /// The declared knobs in a FIXED order, absent ones omitted —
    /// the digest's view. Named pairs rather than positions: adding a
    /// knob later must not silently re-key the ones before it.
    fn declared(&self) -> Vec<(&'static str, usize)> {
        [
            ("file_lines_warn", self.file_lines_warn),
            ("file_lines_fail", self.file_lines_fail),
            ("cognitive_warn", self.cognitive_warn),
            ("fn_lines_warn", self.fn_lines_warn),
            ("fn_lines_fail", self.fn_lines_fail),
            ("ratchet_tolerance", self.ratchet_tolerance),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect()
    }
}

impl ClassCfg {
    /// The class's effective table: its overrides over the global one.
    pub fn effective(&self, global: &Thresholds) -> Thresholds {
        let k = &self.knobs;
        Thresholds {
            file_lines_warn: k.file_lines_warn.unwrap_or(global.file_lines_warn),
            file_lines_fail: k.file_lines_fail.unwrap_or(global.file_lines_fail),
            cognitive_warn: k.cognitive_warn.unwrap_or(global.cognitive_warn),
            fn_lines_warn: k.fn_lines_warn.unwrap_or(global.fn_lines_warn),
            fn_lines_fail: k.fn_lines_fail.unwrap_or(global.fn_lines_fail),
            ..global.clone()
        }
    }
}

/// One length-prefixed field: `<tag>:<len>:<bytes>`. Netstring form,
/// so no field's content can be read as another field's frame.
fn put(buf: &mut Vec<u8>, tag: &str, bytes: &[u8]) {
    buf.extend_from_slice(tag.as_bytes());
    buf.push(b':');
    buf.extend_from_slice(bytes.len().to_string().as_bytes());
    buf.push(b':');
    buf.extend_from_slice(bytes);
}

impl RulesCfg {
    /// The rulepack FINGERPRINT (5.1.0, plan v2.14 (2)), or None when
    /// no class is declared - absence is the state a repo without a
    /// rulepack must keep, byte for byte.
    ///
    /// What it covers is what can move a class's effective lines FROM
    /// THIS DECLARATION: every name, every glob in declaration order
    /// (order IS precedence, so reordering two classes is a different
    /// rulepack), and every knob including the ones this side does not
    /// send on the wire. What it deliberately does NOT cover is the
    /// global [thresholds] a class inherits from - those ride the wire
    /// themselves as `ceilings`/`thresholds` rows and come back in the
    /// reply's knob echo, so they are watched by another road rather
    /// than fenced by this one.
    ///
    /// Every field is written LENGTH-PREFIXED, not merely separated.
    /// The first draft tagged the fields and leaned on fnv1a's NUL
    /// separator, and its own test found the collision immediately: a
    /// class named "a" with glob "b" and a class named "a\0glob\0b"
    /// with no glob produce the identical byte stream. A separator can
    /// only separate what cannot contain it; a length can.
    pub fn digest(&self) -> Option<u64> {
        if self.class.is_empty() {
            return None;
        }
        let mut buf: Vec<u8> = Vec::new();
        for c in &self.class {
            put(&mut buf, "class", c.name.as_bytes());
            for g in &c.globs {
                put(&mut buf, "glob", g.as_bytes());
            }
            for (name, v) in c.knobs.declared() {
                put(&mut buf, "knob", name.as_bytes());
                put(&mut buf, "value", v.to_string().as_bytes());
            }
        }
        Some(crate::score::baseline::fnv1a(&[&buf]))
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn class(name: &str, globs: &[&str], tol: Option<usize>) -> ClassCfg {
        ClassCfg {
            name: name.into(),
            globs: globs.iter().map(|g| (*g).to_string()).collect(),
            knobs: ClassKnobs {
                ratchet_tolerance: tol,
                ..ClassKnobs::default()
            },
        }
    }

    fn digest(classes: Vec<ClassCfg>) -> Option<u64> {
        RulesCfg { class: classes }.digest()
    }

    /// Everything the declaration can say moves the fingerprint, and
    /// the two things that must NOT collide do not. Order is in there
    /// because order IS precedence (the first class whose globs match
    /// wins), so swapping two classes is a different rulepack even
    /// though the same lines are declared.
    #[test]
    fn the_fingerprint_moves_with_every_part_of_the_declaration() {
        let a = class("vendored", &["vendor/**"], None);
        let b = class("fixtures", &["tests/fixtures/**"], None);
        let base = digest(vec![a.clone(), b.clone()]);
        assert!(base.is_some());
        assert_ne!(
            base,
            digest(vec![b.clone(), a.clone()]),
            "declaration order"
        );
        assert_ne!(
            base,
            digest(vec![class("vendor", &["vendor/**"], None), b.clone()]),
            "a class name"
        );
        assert_ne!(
            base,
            digest(vec![class("vendored", &["vendor/*"], None), b.clone()]),
            "a glob"
        );
        assert_ne!(
            base,
            digest(vec![class("vendored", &["vendor/**"], Some(0)), b.clone()]),
            "a knob"
        );
        // and a knob set to zero is not the same as no knob at all —
        // "no growth allowed" and "the global legs" are opposite
        // claims, and a digest that confused them would let one become
        // the other in silence
        assert_ne!(
            digest(vec![class("v", &["v/**"], Some(0))]),
            digest(vec![class("v", &["v/**"], None)]),
            "zero is a declaration, absence is not"
        );
    }

    /// A repo that declares no class has no fingerprint — absence is
    /// the state the fence must leave untouched, byte for byte.
    #[test]
    fn no_rulepack_no_fingerprint() {
        assert_eq!(digest(vec![]), None);
    }

    /// No field's content can be read as another field's frame. This
    /// test failed on the first draft, which separated fields with
    /// fnv1a's NUL instead of measuring them: a class named "a" with
    /// glob "b" hashed identically to one named "a glob b" with no
    /// glob. The length prefix is what makes the encoding injective.
    #[test]
    fn a_name_cannot_impersonate_a_glob() {
        assert_ne!(
            digest(vec![class("a", &["b"], None)]),
            digest(vec![class("a\u{0}glob\u{0}b", &[], None)]),
        );
    }
}
