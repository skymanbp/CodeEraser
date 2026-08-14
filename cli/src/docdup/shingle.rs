//! Wordization and word shingles — ONE implementation for the
//! product coarse filter (3g) AND the offline oracle (F29: the
//! oracle is this same function driven by an O(n²) enumeration; a
//! second implementation would leave the equality gate nothing to
//! hold). Words are maximal alphanumeric runs, lowercased, hashed
//! fnv1a; masked bytes (HTML comment / inline code spans inside md
//! lines) never contribute — the F3 contract that the judge sees
//! nothing the detector masks.

use super::spec::DOC_SHINGLE;
use crate::dedup::tokens::fnv1a;

/// Word-hash sequence of one line under its optional byte mask. A
/// masked byte or a non-alphanumeric char ends the current word; the
/// line end always does.
pub fn line_words(line: &str, mask: Option<&[bool]>, out: &mut Vec<u64>) {
    let mut buf = String::new();
    for (i, c) in line.char_indices() {
        let masked = mask.is_some_and(|m| m[i]);
        if !masked && c.is_alphanumeric() {
            buf.extend(c.to_lowercase());
        } else if !buf.is_empty() {
            out.push(fnv1a(buf.as_bytes()));
            buf.clear();
        }
    }
    if !buf.is_empty() {
        out.push(fnv1a(buf.as_bytes()));
    }
}

/// Sorted deduplicated DOC_SHINGLE-gram set over a word sequence —
/// the Jaccard alphabet. Sequences shorter than the width have no
/// shingles (admission's MIN_DOC_TOKENS floor sits far above it).
pub fn shingle_set(words: &[u64]) -> Vec<u64> {
    shingle_set_k(words, DOC_SHINGLE)
}

/// The width-parameterized form — ONE throat so the instrument-side
/// k-window measurement backing DOC_SHINGLE (spec.rs) runs the exact
/// production shingling at k±1, never a re-implementation.
pub fn shingle_set_k(words: &[u64], k: usize) -> Vec<u64> {
    let mut v = crate::dedup::winnow::kgram_hashes(words, k);
    v.sort_unstable();
    v.dedup();
    v
}

/// The UNSORTED shingle sequence — verbatim runs need order (design
/// §5.3: a common contiguous shingle run of length R spans
/// R + DOC_SHINGLE − 1 words).
pub fn shingle_seq(words: &[u64]) -> Vec<u64> {
    crate::dedup::winnow::kgram_hashes(words, DOC_SHINGLE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_case_folded_alnum_runs_and_masks_erase() {
        let mut a = Vec::new();
        line_words("Hello, WORLD-42!", None, &mut a);
        let mut b = Vec::new();
        line_words("hello world 42", None, &mut b);
        assert_eq!(a, b);
        let line = "keep `code span` keep";
        let mut mask = vec![false; line.len()];
        mask[5..16].fill(true);
        let mut m = Vec::new();
        line_words(line, Some(&mask), &mut m);
        let mut plain = Vec::new();
        line_words("keep keep", None, &mut plain);
        assert_eq!(m, plain);
    }

    #[test]
    fn shingle_set_is_order_free_and_seq_is_not() {
        let w1: Vec<u64> = (0..8).collect();
        let w2: Vec<u64> = (0..8).rev().collect();
        assert_ne!(shingle_seq(&w1), shingle_seq(&w2));
        let mut s1 = shingle_set(&w1);
        s1.sort_unstable();
        assert_eq!(shingle_set(&w1), s1, "already sorted deduped");
        assert_eq!(shingle_seq(&w1).len(), 8 - DOC_SHINGLE + 1);
    }
}
