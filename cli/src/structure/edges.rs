//! File-edge participation (the S3 misplacement input): file-level
//! reference edges roll up, through the tree::dir_of join, to per-file
//! (inside, outside) counts. Pure functions over plain indices — the
//! S2 runner adapts the cached graph wire into `file_dirs`, and NOTHING
//! here judges: whether a file's majority neighborhood living elsewhere
//! is misplacement is the core's call (the ADR-008 boundary), this
//! module only counts. The directed (dirA, dirB) table left with the
//! v2.18 subtraction batch because nothing judged it; it comes back at
//! 7.1.0 (O54) with the modularity axis that does. The INTRA counts do
//! not come back with it: an intra edge adds 1 to `inside` at both of
//! its ends, so `aggregate`'s output already carries twice the intra
//! mass, and the core halves it rather than read a second copy off the
//! wire.

use std::collections::BTreeMap;

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

/// The directed CROSSING dir-edge table: one row per ordered
/// (from, to) pair of DIFFERENT directories, carrying how many file
/// edges run that way (multiplicity preserved — the same multiset the
/// `outside` counts above are drawn from, so the two tables are one
/// graph by construction). BTreeMap iteration gives the strictly
/// ascending (from, to) order the wire demands, free.
pub fn directed(edges: &[(usize, usize)], file_dirs: &[usize]) -> Vec<[u64; 3]> {
    let mut counted: BTreeMap<(usize, usize), u64> = BTreeMap::new();
    for &(a, b) in edges {
        let (da, db) = (file_dirs[a], file_dirs[b]);
        if da != db {
            *counted.entry((da, db)).or_insert(0) += 1;
        }
    }
    counted
        .into_iter()
        .map(|((a, b), n)| [a as u64, b as u64, n])
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/structure/edges.rs"]
mod tests;
