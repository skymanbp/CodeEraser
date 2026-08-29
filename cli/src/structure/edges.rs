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
#[path = "../../tests/unit/structure/edges.rs"]
mod tests;
