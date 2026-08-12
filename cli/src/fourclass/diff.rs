//! Line-level Myers diff over hashed lines. Self-contained so the
//! probe path never shells out to git (ADR-004: git diff is the Stop
//! audit's fallback, not the per-edit engine).
//!
//! Common prefix/suffix are trimmed first — real edits are localized,
//! so the O(ND) core usually sees a small window. `MAX_D` bounds the
//! worst case (a total rewrite of a huge file); beyond it every
//! trimmed line is reported changed, which over-counts novel/deleted
//! but never invents a move — the degradation is flagged.

/// Indices (0-based) of changed lines on each side.
pub struct DiffLines {
    pub removed: Vec<usize>,
    pub added: Vec<usize>,
    /// True when the edit-distance cap tripped and the trimmed window
    /// was classified wholesale instead of minimally.
    pub degraded: bool,
}

const MAX_D: usize = 3000;

pub fn diff(a: &[u64], b: &[u64]) -> DiffLines {
    let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let (a_t, b_t) = (&a[prefix..], &b[prefix..]);
    let suffix = a_t
        .iter()
        .rev()
        .zip(b_t.iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (a_t, b_t) = (&a_t[..a_t.len() - suffix], &b_t[..b_t.len() - suffix]);

    // One side empty after trimming = pure insertion/deletion, whose
    // minimal script IS every remaining line of the other side —
    // exact by construction, so it must not ride the D-bounded search
    // (which flags any window longer than MAX_D degraded: first hit
    // was ripgrep 082245dad creating the 7,625-line flags/defs.rs).
    let (removed, added, degraded) = if a_t.is_empty() || b_t.is_empty() {
        ((0..a_t.len()).collect(), (0..b_t.len()).collect(), false)
    } else {
        match myers(a_t, b_t) {
            Some((r, ad)) => (r, ad, false),
            None => ((0..a_t.len()).collect(), (0..b_t.len()).collect(), true),
        }
    };
    DiffLines {
        removed: removed.iter().map(|i| i + prefix).collect(),
        added: added.iter().map(|i| i + prefix).collect(),
        degraded,
    }
}

/// Greedy forward Myers recording one furthest-reach vector per round;
/// None when the edit distance exceeds MAX_D.
#[allow(clippy::type_complexity)] // the pair IS the result: (removed, added) index lists
fn myers(a: &[u64], b: &[u64]) -> Option<(Vec<usize>, Vec<usize>)> {
    let max = (a.len() + b.len()).min(MAX_D);
    let off = max + 1; // k ∈ [-d, d] stored at k + off
    let mut v = vec![0usize; 2 * off + 1];
    let mut trace: Vec<Vec<usize>> = Vec::new();
    for d in 0..=max {
        trace.push(v.clone());
        let mut k = -(d as isize);
        while k <= d as isize {
            let (x, y) = step(&mut v, k, d, off, (a, b));
            if x >= a.len() && y >= b.len() {
                return Some(backtrack(&trace, d, off, (a.len(), b.len())));
            }
            k += 2;
        }
    }
    None
}

/// One diagonal step of round `d`: pick the better predecessor, slide
/// the snake, record the furthest reach. Returns the (x, y) reached.
fn step(
    v: &mut [usize],
    k: isize,
    d: usize,
    off: usize,
    (a, b): (&[u64], &[u64]),
) -> (usize, usize) {
    let ki = (k + off as isize) as usize;
    let mut x = if k == -(d as isize) || (k != d as isize && v[ki - 1] < v[ki + 1]) {
        v[ki + 1]
    } else {
        v[ki - 1] + 1
    };
    let mut y = (x as isize - k) as usize;
    while x < a.len() && y < b.len() && a[x] == b[y] {
        x += 1;
        y += 1;
    }
    v[ki] = x;
    (x, y)
}

/// Walk the round snapshots backwards from (n, m): each round is one
/// insertion or deletion followed by a (possibly empty) snake.
fn backtrack(
    trace: &[Vec<usize>],
    d_final: usize,
    off: usize,
    end: (usize, usize),
) -> (Vec<usize>, Vec<usize>) {
    let (mut x, mut y) = end;
    let (mut removed, mut added) = (Vec::new(), Vec::new());
    for d in (1..=d_final).rev() {
        let v = &trace[d];
        let k = x as isize - y as isize;
        let ki = (k + off as isize) as usize;
        let down = k == -(d as isize) || (k != d as isize && v[ki - 1] < v[ki + 1]);
        let prev_k = if down { k + 1 } else { k - 1 };
        let prev_x = v[(prev_k + off as isize) as usize];
        let prev_y = (prev_x as isize - prev_k) as usize;
        if down {
            added.push(prev_y); // b[prev_y] inserted
        } else {
            removed.push(prev_x); // a[prev_x] deleted
        }
        x = prev_x;
        y = prev_y;
    }
    removed.reverse();
    added.reverse();
    (removed, added)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(a: &[u64], b: &[u64]) -> (Vec<usize>, Vec<usize>) {
        let d = diff(a, b);
        assert!(!d.degraded);
        (d.removed, d.added)
    }

    #[test]
    fn identical_inputs_have_no_changes() {
        assert_eq!(run(&[1, 2, 3], &[1, 2, 3]), (vec![], vec![]));
        assert_eq!(run(&[], &[]), (vec![], vec![]));
    }

    #[test]
    fn pure_insertion_and_deletion() {
        assert_eq!(run(&[1, 3], &[1, 2, 3]), (vec![], vec![1]));
        assert_eq!(run(&[1, 2, 3], &[1, 3]), (vec![1], vec![]));
        assert_eq!(run(&[], &[7, 8]), (vec![], vec![0, 1]));
        assert_eq!(run(&[7, 8], &[]), (vec![0, 1], vec![]));
    }

    #[test]
    fn one_side_empty_is_exact_beyond_the_cap() {
        // a pure creation/deletion larger than MAX_D is still exact —
        // only the bounded SEARCH degrades, never the trivial script
        let big: Vec<u64> = (0..(MAX_D as u64 + 10)).collect();
        let d = diff(&[], &big);
        assert!(!d.degraded);
        assert_eq!((d.removed.len(), d.added.len()), (0, big.len()));
        let d = diff(&big, &[]);
        assert!(!d.degraded);
        assert_eq!((d.removed.len(), d.added.len()), (big.len(), 0));
    }

    #[test]
    fn replacement_counts_both_sides() {
        let (r, a) = run(&[1, 2, 3], &[1, 9, 3]);
        assert_eq!((r, a), (vec![1], vec![1]));
    }

    #[test]
    fn relocation_reports_one_removal_one_addition() {
        // 5 moved from front to back — minimal script is one del + one add.
        let (r, a) = run(&[5, 1, 2, 3], &[1, 2, 3, 5]);
        assert_eq!(r.len(), 1);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn counts_are_minimal_against_lcs_on_small_cases() {
        // |removed| = n - LCS, |added| = m - LCS for adversarial small
        // inputs (rotation, reversal, duplicate swap, alternation),
        // derived programmatically and LCS'd by brute force.
        let base: Vec<u64> = (1..=5).collect();
        let rotated = [&base[2..], &base[..2]].concat();
        let reversed: Vec<u64> = base.iter().rev().copied().collect();
        let dupes: Vec<u64> = vec![1, 1, 2, 2];
        let dupes_swapped = [&dupes[2..], &dupes[..2]].concat();
        let alt: Vec<u64> = (0..5).map(|i| 1 + i % 2).collect();
        let alt_flipped: Vec<u64> = alt.iter().map(|v| 3 - v).collect();
        for (a, b) in [
            (&base, &rotated),
            (&base, &reversed),
            (&dupes, &dupes_swapped),
            (&alt, &alt_flipped),
        ] {
            let lcs = lcs_len(a, b);
            let (r, ad) = run(a, b);
            assert_eq!(r.len(), a.len() - lcs, "removed for {a:?} vs {b:?}");
            assert_eq!(ad.len(), b.len() - lcs, "added for {a:?} vs {b:?}");
        }
    }

    fn lcs_len(a: &[u64], b: &[u64]) -> usize {
        let mut t = vec![vec![0usize; b.len() + 1]; a.len() + 1];
        for i in 1..=a.len() {
            for j in 1..=b.len() {
                t[i][j] = if a[i - 1] == b[j - 1] {
                    t[i - 1][j - 1] + 1
                } else {
                    t[i - 1][j].max(t[i][j - 1])
                };
            }
        }
        t[a.len()][b.len()]
    }
}
