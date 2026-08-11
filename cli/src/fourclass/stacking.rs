//! Stacking evidence for the M4 judgment rule: unit keys newly
//! DUPLICATED on the after side, shipped as hashes only (ADR-002
//! A6). Split from batch.rs when the attack-review fixes pushed that
//! file past the 300-line budget (E01: split before exemption).

use super::batch::PairInput;
use super::units;
use crate::dedup::tokens::fnv1a;
use std::collections::HashMap;

/// Unit keys whose after-side count rises to >= 2. Only TOP-LEVEL
/// named units count: a method nested in two different classes
/// shares its flat key legitimately, and an anonymous closure has no
/// stacking identity at all — both were measured false-positive
/// sources on the real-edit corpus (contracts/eval/
/// fpr-fourclass-v1.json flagged 8/600 before this scoping, 0/600
/// after). "impl " units are CONTAINERS (they exist so methods are
/// not top-level), never stacking identities themselves: a type's
/// inherent and trait impls, or split inherent impls, coexist in
/// perfectly normal Rust — the FPR replay flagged exactly that shape
/// when they were counted (attack review F7).
pub fn dup_units(input: &PairInput) -> Vec<u64> {
    let count = |text: &str| -> HashMap<String, usize> {
        let all = units::segments(text, input.lang);
        let top_level = |u: &units::Unit| {
            !all.iter().any(|v| {
                (v.start_line < u.start_line && u.end_line <= v.end_line)
                    || (v.start_line <= u.start_line && u.end_line < v.end_line)
            })
        };
        all.iter()
            .filter(|u| {
                top_level(u) && !u.key.starts_with("(anonymous)") && !u.key.starts_with("impl ")
            })
            .fold(HashMap::new(), |mut m, u| {
                *m.entry(u.key.clone()).or_default() += 1;
                m
            })
    };
    let before = count(input.before);
    let mut out: Vec<u64> = count(input.after)
        .into_iter()
        .filter(|(k, n)| *n >= 2 && *n > before.get(k).copied().unwrap_or(0))
        .map(|(k, _)| fnv1a(k.as_bytes()))
        .collect();
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::lang::Lang;

    /// Red/green for the stacking evidence. The green direction: a
    /// genuine top-level duplicate fires. The red direction, one row
    /// per measured or attack-reviewed false-positive shape: methods
    /// under different Python classes, anonymous Rust closures
    /// (fpr-fourclass 8/600 before the top-level scoping), methods
    /// under different Rust impls, impl containers themselves
    /// (inherent + trait impl of one type — the FPR replay caught the
    /// unqualified key colliding), and Go methods on different
    /// receivers (attack review F7).
    #[test]
    fn dup_evidence_is_top_level_and_named() {
        let real = PairInput {
            before: "fn one() {}\n",
            after: "fn one() {}\nfn work(a: i32) -> i32 { a }\nfn work(a: i32) -> i32 { a + 1 }\n",
            lang: Lang::Rust,
        };
        assert_eq!(dup_units(&real).len(), 1, "top-level duplicate is evidence");
        let none: [(&str, Lang, &str); 5] = [
            (
                "class A:\n    def add(self, x):\n        pass\n\nclass B:\n    def add(self, x):\n        pass\n",
                Lang::Python,
                "cross-class methods",
            ),
            (
                "fn go() {\n    let a = |x: i32| x;\n    let b = |x: i32| x + 1;\n}\n",
                Lang::Rust,
                "anonymous closures",
            ),
            (
                "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n",
                Lang::Rust,
                "cross-impl methods",
            ),
            (
                "impl Foo {\n    fn go(&self) {}\n}\nimpl Advisor for Foo {\n    fn advise(&self) {}\n}\n",
                Lang::Rust,
                "impl containers",
            ),
            (
                "func (t T) add(x int) {}\nfunc (u U) add(x int) {}\n",
                Lang::Go,
                "cross-receiver Go methods",
            ),
        ];
        for (after, lang, what) in none {
            let input = PairInput {
                before: "",
                after,
                lang,
            };
            assert_eq!(
                dup_units(&input),
                Vec::<u64>::new(),
                "{what} are not evidence"
            );
        }
    }
}
