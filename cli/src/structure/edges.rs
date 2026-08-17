//! Directory-edge aggregation (S2 orthogonality / S3 misplacement
//! inputs): file-level reference edges roll up to directed
//! (dirA, dirB) counts plus per-file inside/outside splits. Pure
//! functions over plain indices — the S2 runner adapts the cached
//! graph wire into `file_dirs`, and NOTHING here judges: whether a
//! file's majority neighborhood living elsewhere is misplacement is
//! the core's call (the ADR-008 boundary), this module only counts.

use std::collections::BTreeMap;

/// The rolled-up reference geometry.
pub struct DirEdges {
    /// Directed cross-directory counts (a, b, refs), a ≠ b,
    /// ascending — the S2 modularity inputs.
    pub inter: Vec<(usize, usize, u32)>,
    /// Per-directory intra-reference counts, indexed by dir id.
    pub intra: Vec<u32>,
    /// Per-FILE (inside, outside) reference participation — both
    /// endpoints of every edge count, so a file's row reflects its
    /// dependencies AND its dependents (the S3 axis input).
    pub files: Vec<[u32; 2]>,
}

/// Roll `edges` (file-index pairs) up through `file_dirs` (file
/// index → owning dir id, the tree::dir_of join) over `dir_count`
/// directories.
pub fn aggregate(edges: &[(usize, usize)], file_dirs: &[usize], dir_count: usize) -> DirEdges {
    let mut inter: BTreeMap<(usize, usize), u32> = BTreeMap::new();
    let mut intra = vec![0u32; dir_count];
    let mut files = vec![[0u32; 2]; file_dirs.len()];
    for &(a, b) in edges {
        let (da, db) = (file_dirs[a], file_dirs[b]);
        if da == db {
            intra[da] += 1;
            files[a][0] += 1;
            files[b][0] += 1;
        } else {
            *inter.entry((da, db)).or_insert(0) += 1;
            files[a][1] += 1;
            files[b][1] += 1;
        }
    }
    DirEdges {
        inter: inter.into_iter().map(|((a, b), n)| (a, b, n)).collect(),
        intra,
        files,
    }
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
        let g = aggregate(&edges, &file_dirs, 2);
        assert_eq!(g.intra, vec![1, 0], "one intra edge inside dir 0");
        assert_eq!(
            g.inter,
            vec![(0, 1, 2), (1, 0, 1)],
            "direction preserved, ascending keys"
        );
        // file 0: intra 0→1 (+1 inside) + inter 0→2, 2→0 (+2 outside)
        // file 1: intra (+1 inside) + inter 1→2 (+1 outside)
        // file 2: three inter touches, zero inside
        assert_eq!(g.files, vec![[1, 2], [1, 1], [0, 3]]);
    }
}
