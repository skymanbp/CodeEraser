//! slug_hash ↔ slug_set coupling battery (the M5-close staleness
//! repayment's one unpinned invariant): the hashed projection and
//! the consulted projection must MOVE TOGETHER — a refactor that
//! hashes the raw text, the unmasked text, or a deduplicated set
//! keeps every e2e green while quietly re-opening (or degrading)
//! the cross-file staleness repayment. Their divergence is the only
//! way that debt returns, so it is pinned here as a table.

use super::{slug_hash, slug_set};

#[test]
fn hash_moves_exactly_when_the_consulted_projection_moves() {
    let pairs: &[(&str, &str, &str)] = &[
        ("body-only edit holds the hash", "# A\nx\n", "# A\ny\n"),
        ("heading edit moves it", "# Alpha\nbody\n", "# Beta\nbody\n"),
        (
            "fenced pseudo-headings are masked on both sides",
            "# A\n```\n# Hidden\n```\n",
            "# A\n```\n# Other\n```\n",
        ),
        ("a duplicate heading enters as -N", "# A\n", "# A\n# A\n"),
    ];
    for (why, a, b) in pairs {
        assert_eq!(
            slug_hash(a) == slug_hash(b),
            slug_set(a) == slug_set(b),
            "projection coupling broke: {why}"
        );
    }
    assert_eq!(slug_set("# A\n# A\n"), ["a", "a-1"], "the -N suffix rule");
}
