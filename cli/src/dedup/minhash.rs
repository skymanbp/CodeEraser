//! Deterministic MinHash + LSH banding — the ONE implementation both
//! the T3 structural candidate source (S4) and the docdup coarse
//! filter consume (design vol.1 §1.1: a second copy would let the
//! two estimators drift). No RNG, no clocks (F21): permutation i IS
//! the salt — element bytes followed by i.to_le_bytes(), hashed with
//! the repo's one fnv1a.

use super::tokens::fnv1a;

/// sig[i] = min over x in `set` of fnv1a(x ‖ i). An empty set yields
/// u64::MAX rows — such a signature equals another only if that one
/// is empty too, and empty-vs-empty pairs are the caller's admission
/// problem, not a hash accident.
pub fn signature(set: &[u64], perms: usize) -> Vec<u64> {
    (0..perms as u32)
        .map(|i| {
            set.iter()
                .map(|x| {
                    let mut bytes = [0u8; 12];
                    bytes[..8].copy_from_slice(&x.to_le_bytes());
                    bytes[8..].copy_from_slice(&i.to_le_bytes());
                    fnv1a(&bytes)
                })
                .min()
                .unwrap_or(u64::MAX)
        })
        .collect()
}

/// (band index, band hash) for every band of `sig` under a bands×rows
/// split. Two signatures share a bucket iff some band's rows agree —
/// the LSH candidate condition.
pub fn band_keys(sig: &[u64], bands: usize, rows: usize) -> Vec<(usize, u64)> {
    assert_eq!(
        sig.len(),
        bands * rows,
        "bands*rows must cover the signature"
    );
    (0..bands)
        .map(|b| {
            let mut bytes = Vec::with_capacity(8 * rows);
            for v in &sig[b * rows..(b + 1) * rows] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            (b, fnv1a(&bytes))
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/dedup/minhash.rs"]
mod tests;
