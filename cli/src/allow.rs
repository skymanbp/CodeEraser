//! The `ce:allow(<tag>)` claim, parsed in ONE place (plan v2.17 L
//! round piece (4); sealed criterion §3.2 bit 10). Three lifetimes
//! read it — docdup over a comment segment's lines, deadcode over a
//! whole file for the entry role, the mention advisory over a whole
//! file at wire time — and the first two each carried a parser of
//! their own. One grammar now, the plan's own written form
//! `ce:allow(<rule>) -- <why>` (plan :76, §4.1): the tag, blanks,
//! `--`, a blank, and a why that is not blank on the marker's own
//! line. A bare marker claims NOTHING ("no why = a violation") — the
//! rule the docdup exemption established and deadcode transplanted,
//! stated once so the discipline cannot drift twice.
//!
//! Against the two parsers it replaces, every seam is named: docdup
//! let text sit between marker and `--` and took `--why` with no
//! blank (both narrowed); deadcode took an empty why (narrowed) and
//! spelled the blank after `--` as one space only, so `--<TAB>why`
//! now claims where it did not (the one widening — a file more that
//! is exempt from the dead verdict, never a file less).

/// Blank = space or tab; a why must be separated from `--` by one.
const BLANK: [char; 2] = [' ', '\t'];

/// Whether `text` carries a why-bearing `<tag> -- <why>` claim.
pub fn allow_claim(text: &str, tag: &str) -> bool {
    text.match_indices(tag).any(|(at, hit)| {
        let tail = text[at + hit.len()..].trim_start_matches(BLANK);
        let why = tail
            .strip_prefix("--")
            .and_then(|why| why.lines().next())
            .unwrap_or("");
        why.starts_with(BLANK) && !why.trim().is_empty()
    })
}

#[cfg(test)]
#[path = "../tests/unit/allow.rs"]
mod tests;
