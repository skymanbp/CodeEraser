//! File-edge participation (the S3 misplacement input): file-level
//! reference edges roll up, through the tree::dir_of join, to per-file
//! (inside, outside) counts. Pure functions over plain indices — the
//! S2 runner adapts the cached graph wire into `file_dirs`, and NOTHING
//! here judges: whether a file's majority neighborhood living elsewhere
//! is misplacement is the core's call (the ADR-008 boundary), this
//! module only counts. The directed (dirA, dirB) table and the intra
//! counts once kept beside this were never on the wire — Axes.hs rules
//! an unjudged table dead freight, not a reservation — and left with
//! the v2.18 subtraction batch.

/// Per-FILE (inside, outside) reference participation — both
/// endpoints of every edge count, so a file's row reflects its
/// dependencies AND its dependents.
pub fn aggregate(edges: &[(usize, usize)], file_dirs: &[usize]) -> Vec<[u32; 2]> {
    let mut files = vec![[0u32; 2]; file_dirs.len()];
    for &(a, b) in edges {
        let side = usize::from(file_dirs[a] != file_dirs[b]);
        files[a][side] += 1;
        files[b][side] += 1;
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One synthetic geometry, every count hand-checked: files 0,1
    /// live in dir 0, file 2 in dir 1; edges 0→1 (intra), 0→2 and
    /// 1→2 (inter), 2→0 (inter, reverse direction stays distinct).
    #[test]
    fn aggregate_counts_a_small_geometry_by_hand() {
        let file_dirs = [0, 0, 1];
        let edges = [(0, 1), (0, 2), (1, 2), (2, 0)];
        // file 0: intra 0→1 (+1 inside) + inter 0→2, 2→0 (+2 outside)
        // file 1: intra (+1 inside) + inter 1→2 (+1 outside)
        // file 2: three inter touches, zero inside
        assert_eq!(aggregate(&edges, &file_dirs), vec![[1, 2], [1, 1], [0, 3]]);
    }
}
