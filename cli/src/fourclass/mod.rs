//! F4 update-supervision four-classification (plan §4.3): the L1 rung
//! — Myers line diff + tree-sitter function-boundary alignment.
//!
//! Move semantics match the ground-truth convention (labels-v1.json):
//! a changed line is *moved* when its content — leading/trailing
//! whitespace ignored, matching git's allow-indentation-change — also
//! appears on the opposite side of the diff, and the line is
//! *significant* (carries at least one alphanumeric character). Sides
//! are marked independently, so counts may be unbalanced, exactly as
//! git marks them. Blank and pure-punctuation lines belong to no
//! symbol, so they can never "move" — the artifact class the ground
//! truth review corrected out.
//!
//! The function-boundary half attributes every moved line to the unit
//! it left and the unit it joined (the guard's 指回位置), and
//! summarizes intact unit relocations.

pub mod diff;
pub mod units;

use crate::scan::lang::Lang;
use std::collections::HashSet;
use units::Unit;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FourClass {
    pub added_novel: usize,
    pub added_moved: usize,
    pub removed_deleted: usize,
    pub removed_moved: usize,
}

/// One moved line with its unit attribution. `unit` is the owning
/// unit's key on the side the line sits on (None = file top level).
#[derive(Debug, PartialEq, Eq)]
pub struct MovedLine {
    pub line: usize, // 1-based, on its own side
    pub removed: bool,
    pub unit: Option<String>,
}

pub struct Classification {
    pub counts: FourClass,
    pub moved: Vec<MovedLine>,
    /// Unit keys present on both sides whose changed lines are all
    /// moves — the "function relocated intact" summary.
    pub relocated_units: Vec<String>,
    pub degraded: bool,
}

pub fn classify(before: &str, after: &str, lang: Lang) -> Classification {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let d = diff::diff(&hash_lines(&a), &hash_lines(&b));

    let removed_sig: HashSet<&str> = sig_contents(&a, &d.removed);
    let added_sig: HashSet<&str> = sig_contents(&b, &d.added);
    let before_units = units::segments(before, lang);
    let after_units = units::segments(after, lang);

    let mut counts = FourClass::default();
    let mut moved = Vec::new();
    for &i in &d.removed {
        if significant(a[i]) && added_sig.contains(a[i].trim()) {
            counts.removed_moved += 1;
            moved.push(moved_line(i, true, &before_units));
        } else {
            counts.removed_deleted += 1;
        }
    }
    for &j in &d.added {
        if significant(b[j]) && removed_sig.contains(b[j].trim()) {
            counts.added_moved += 1;
            moved.push(moved_line(j, false, &after_units));
        } else {
            counts.added_novel += 1;
        }
    }
    let relocated_units = relocated(&moved, &before_units, &after_units, &d);
    Classification {
        counts,
        moved,
        relocated_units,
        degraded: d.degraded,
    }
}

fn hash_lines(lines: &[&str]) -> Vec<u64> {
    use std::hash::{DefaultHasher, Hash, Hasher};
    lines
        .iter()
        .map(|l| {
            let mut h = DefaultHasher::new();
            l.hash(&mut h);
            h.finish()
        })
        .collect()
}

/// A line can carry move identity only if something in it names
/// anything — blank lines and bare punctuation match anywhere and
/// mean nothing.
fn significant(line: &str) -> bool {
    line.chars().any(char::is_alphanumeric)
}

fn sig_contents<'s>(lines: &[&'s str], changed: &[usize]) -> HashSet<&'s str> {
    changed
        .iter()
        .map(|&i| lines[i].trim())
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect()
}

fn moved_line(idx: usize, removed: bool, side_units: &[Unit]) -> MovedLine {
    MovedLine {
        line: idx + 1,
        removed,
        unit: units::owner(side_units, idx + 1).map(|u| u.key.clone()),
    }
}

/// A unit relocated intact when it exists on both sides and every
/// changed line inside either span is a move (nothing was edited,
/// only position changed).
fn relocated(
    moved: &[MovedLine],
    before_units: &[Unit],
    after_units: &[Unit],
    d: &diff::DiffLines,
) -> Vec<String> {
    let moved_of = |removed: bool, key: &str| {
        moved
            .iter()
            .filter(|m| m.removed == removed && m.unit.as_deref() == Some(key))
            .count()
    };
    let changed_in = |unit: &Unit, changed: &[usize]| {
        changed
            .iter()
            .filter(|&&i| unit.start_line <= i + 1 && i < unit.end_line)
            .count()
    };
    let mut out = Vec::new();
    for bu in before_units {
        let Some(au) = after_units.iter().find(|u| u.key == bu.key) else {
            continue;
        };
        let (rm, ad) = (changed_in(bu, &d.removed), changed_in(au, &d.added));
        if rm + ad > 0 && moved_of(true, &bu.key) == rm && moved_of(false, &au.key) == ad {
            out.push(bu.key.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocation_with_indent_change_is_moved_and_attributed() {
        let before = "def f():\n    x = compute()\n    return x\n\ndef g():\n    pass\n";
        let after =
            "def f():\n    if ok:\n        x = compute()\n    return x\n\ndef g():\n    pass\n";
        let c = classify(before, after, Lang::Python);
        assert_eq!(c.counts.added_moved, 1, "indented re-add of x=compute()");
        assert_eq!(c.counts.removed_moved, 1);
        assert_eq!(c.counts.added_novel, 1); // the new `if ok:` line
        let m = c.moved.iter().find(|m| !m.removed).unwrap();
        assert_eq!(m.unit.as_deref(), Some("f/0"));
    }

    #[test]
    fn blank_lines_never_move() {
        let before = "a = 1\n\nb = 2\n";
        let after = "b = 2\n\nc = 3\n";
        let c = classify(before, after, Lang::Python);
        assert_eq!(c.counts.added_moved + c.counts.removed_moved, 0);
    }

    #[test]
    fn intact_function_relocation_is_summarized() {
        let before = "def a():\n    return 1\n\ndef b():\n    return 2\n";
        let after = "def b():\n    return 2\n\ndef a():\n    return 1\n";
        let c = classify(before, after, Lang::Python);
        assert!(
            c.relocated_units.contains(&"a/0".to_string())
                || c.relocated_units.contains(&"b/0".to_string()),
            "one of the swapped functions must be reported relocated, got {:?}",
            c.relocated_units
        );
        // The swap repositions one BLANK separator line; per the
        // ground-truth convention blanks never move, so it lands in
        // novel/deleted while both code lines are moved.
        assert_eq!(c.counts.added_novel, 1);
        assert_eq!(c.counts.added_moved, 2);
        assert_eq!(c.counts.removed_moved, 2);
    }
}
