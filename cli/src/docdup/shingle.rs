//! Wordization and word shingles — ONE implementation for the
//! product coarse filter (3g) AND the offline oracle (F29: the
//! oracle is this same function driven by an O(n²) enumeration; a
//! second implementation would leave the equality gate nothing to
//! hold). Words are maximal alphanumeric-plus-combining-mark runs,
//! lowercased, NFC-composed, hashed fnv1a; masked bytes (HTML
//! comment / inline code spans inside md lines) never contribute —
//! the F3 contract that the judge sees nothing the detector masks.

use super::spec::DOC_SHINGLE;
use crate::dedup::tokens::fnv1a;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Word-hash sequence of one line under its optional byte mask. A
/// masked byte or a non-alphanumeric char ends the current word; the
/// line end always does. A combining mark (General_Category=Mark)
/// does NOT end a word, and the finished word is NFC-composed before
/// hashing. Both halves are needed: "café" spelled NFC is one
/// alphanumeric run, while the NFD spelling breaks at the mark into
/// "cafe" and drops it — so identical prose in two canonical
/// encodings produced two different word streams, two disjoint
/// shingle sets, and a silent FALSE NEGATIVE nothing counted (review
/// 2026-08-19).
pub fn line_words(line: &str, mask: Option<&[bool]>, out: &mut Vec<u64>) {
    let mut buf = String::new();
    for (i, c) in line.char_indices() {
        let masked = mask.is_some_and(|m| m[i]);
        let carries = c.is_alphanumeric() || (!buf.is_empty() && is_combining_mark(c));
        if !masked && carries {
            buf.extend(c.to_lowercase());
        } else if !buf.is_empty() {
            push_word(&mut buf, out);
        }
    }
    if !buf.is_empty() {
        push_word(&mut buf, out);
    }
}

/// Hash one finished word in NFC and clear the buffer — the ONE
/// throat to fnv1a, so no canonically-equivalent spelling of a word
/// can reach the hash as different bytes.
fn push_word(buf: &mut String, out: &mut Vec<u64>) {
    out.push(fnv1a(buf.as_str().nfc().collect::<String>().as_bytes()));
    buf.clear();
}

/// Sorted deduplicated DOC_SHINGLE-gram set over a word sequence —
/// the Jaccard alphabet. Sequences shorter than the width have no
/// shingles (admission's MIN_DOC_TOKENS floor sits far above it).
pub fn shingle_set(words: &[u64]) -> Vec<u64> {
    shingle_set_k(words, DOC_SHINGLE)
}

/// The width-parameterized throat. It stays `pub` for the RETIRED
/// k-window scout (eval_docdup_oracle, v0.5.0 — a revival per EVAL-SET.md
/// 「再生成」 runs the exact production shingling at k±1, never a copy).
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

    /// One unmasked pass — the probe both equivalence tests share
    /// (their twin stanzas were a clone block).
    fn words(line: &str) -> Vec<u64> {
        let mut out = Vec::new();
        line_words(line, None, &mut out);
        out
    }

    #[test]
    fn words_are_case_folded_alnum_runs_and_masks_erase() {
        assert_eq!(words("Hello, WORLD-42!"), words("hello world 42"));
        let line = "keep `code span` keep";
        let mut mask = vec![false; line.len()];
        mask[5..16].fill(true);
        let mut m = Vec::new();
        line_words(line, Some(&mask), &mut m);
        let mut plain = Vec::new();
        line_words("keep keep", None, &mut plain);
        assert_eq!(m, plain);
    }

    /// Canonical equivalence: the same prose typed NFC and NFD must
    /// yield the SAME word hashes, and the combining mark must not
    /// split a word in two. Unnormalized, the decomposed line gave
    /// three words to the composed line's two.
    #[test]
    fn composed_and_decomposed_prose_hash_alike() {
        let nfc = words("caf\u{e9} na\u{ef}ve");
        assert_eq!(
            nfc,
            words("cafe\u{301} nai\u{308}ve"),
            "canonical equivalents hash alike"
        );
        assert_eq!(nfc.len(), 2, "a mark must not end the word it sits on");
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
