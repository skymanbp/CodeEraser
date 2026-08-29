//! Winnowing over the normalized token stream (Schleimer et al.,
//! SIGMOD'03, Fig. 5 "robust winnowing"): k-gram Rabin-Karp hashes,
//! then per window of `window` consecutive k-grams keep the minimum
//! (rightmost on ties, recorded once). Correctness bound: any match of
//! at least t = window + kgram - 1 tokens shares ≥ 1 fingerprint; no
//! match shorter than kgram tokens is ever reported.

use super::Params;

/// A selected fingerprint: `hash` covers tokens `[start, start+kgram)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint {
    pub hash: u64,
    pub start: usize,
}

const BASE: u64 = 1_000_003;

/// Rolling k-gram hashes of the token-hash sequence (wrapping u64
/// arithmetic; standard Rabin-Karp update). pub(crate): docdup's
/// word shingles ride the SAME rolling hash (design §5.3, F29 — the
/// offline oracle and the product filter must share one
/// implementation, and a second Rabin-Karp would fork it).
pub(crate) fn kgram_hashes(tokens: &[u64], k: usize) -> Vec<u64> {
    // k == 0 is not a degenerate answer, it is arithmetic that cannot
    // run: `k as u32 - 1` below underflows (debug panic / a garbage
    // `top` in release). The invariant lives here, where the
    // subtraction is, not in every caller (review 2026-08-19).
    if k == 0 || tokens.len() < k {
        return Vec::new();
    }
    let top = BASE.wrapping_pow(k as u32 - 1);
    let mut hashes = Vec::with_capacity(tokens.len() - k + 1);
    let mut h: u64 = 0;
    for t in &tokens[..k] {
        h = h.wrapping_mul(BASE).wrapping_add(*t);
    }
    hashes.push(h);
    for i in k..tokens.len() {
        h = h
            .wrapping_sub(tokens[i - k].wrapping_mul(top))
            .wrapping_mul(BASE)
            .wrapping_add(tokens[i]);
        hashes.push(h);
    }
    hashes
}

/// Robust winnowing: in each window take the minimal hash, preferring
/// the rightmost on ties, and record a position only once.
pub fn fingerprints(tokens: &[u64], p: Params) -> Vec<Fingerprint> {
    // window == 0 underflows `p.window - 1` in the window count below
    // (same reason as kgram_hashes' k == 0): a zero-width window
    // selects nothing, so say so at the entry (review 2026-08-19).
    if p.window == 0 {
        return Vec::new();
    }
    let grams = kgram_hashes(tokens, p.kgram);
    if grams.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Fingerprint> = Vec::new();
    let mut last_recorded = usize::MAX;
    let windows = grams.len().saturating_sub(p.window - 1).max(1);
    for w in 0..windows {
        let end = (w + p.window).min(grams.len());
        let mut min_idx = w;
        for i in w..end {
            if grams[i] <= grams[min_idx] {
                min_idx = i; // <= keeps the rightmost minimum
            }
        }
        if min_idx != last_recorded {
            out.push(Fingerprint {
                hash: grams[min_idx],
                start: min_idx,
            });
            last_recorded = min_idx;
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/dedup/winnow.rs"]
mod tests;
