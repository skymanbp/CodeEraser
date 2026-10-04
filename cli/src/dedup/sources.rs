//! The three index-bound T3 candidate sources (design vol.2 §4.2), one
//! generator walk each — split from candidates.rs at the 300-line
//! dogfood gate. Every source emits canonical same-language unit pairs
//! through one push throat; everything either side sheds (self pairs,
//! cross language, lines outside any admitted unit) is tallied there.
//! The fourth, S2 (same key in different files), is a function of the
//! units alone and runs in the core since plan v2.33 W3
//! (candidate_wire.rs).

use super::candidates::{HOT_GROUP_CAP, LSH_SHAPE, Tally, Unit};
use super::index::Instance;
use super::pairs::GroupEvent;
use super::{Params, minhash, pairs, walkidx};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct Gen<'u> {
    units: &'u [Unit],
    by_file: BTreeMap<&'u str, Vec<usize>>,
    pub(super) tally: Tally,
}

/// Each file's unit ids, in unit order.
pub(crate) fn by_file(units: &[Unit]) -> BTreeMap<&str, Vec<usize>> {
    let mut out: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (id, u) in units.iter().enumerate() {
        out.entry(&u.path).or_default().push(id);
    }
    out
}

impl<'u> Gen<'u> {
    pub(super) fn new(units: &'u [Unit]) -> Self {
        Gen {
            units,
            by_file: by_file(units),
            tally: Tally::default(),
        }
    }

    /// The three sources merged into (pair → source bits): bit i means
    /// candidates::SOURCES[i] found the pair (S1 bit 0, S3 bit 2, S4
    /// bit 3; S2's bit 1 is the core's).
    pub(super) fn union(
        &mut self,
        root: &Path,
        instances: &[Instance],
    ) -> Result<BTreeMap<(usize, usize), u8>> {
        let sets = [
            (0, self.near_pairs(root, instances)?),
            (2, self.fingerprint_pairs(instances)),
            (3, self.structural_pairs()),
        ];
        let mut union: BTreeMap<(usize, usize), u8> = BTreeMap::new();
        for (bit, set) in sets {
            for p in set {
                *union.entry(p).or_insert(0) |= 1 << bit;
            }
        }
        Ok(union)
    }

    /// Innermost admitted unit containing `line`, if any.
    fn owner(&self, file: &str, line: i64) -> Option<usize> {
        self.by_file
            .get(file)?
            .iter()
            .copied()
            .filter(|&id| self.units[id].start_line <= line && line <= self.units[id].end_line)
            .min_by_key(|&id| self.units[id].end_line - self.units[id].start_line)
    }

    /// Canonicalize, gate (self pair / cross language), tally, insert.
    fn push(&mut self, set: &mut BTreeSet<(usize, usize)>, x: usize, y: usize, src: &str) {
        if x == y {
            self.tally.self_pair_dropped += 1;
            return;
        }
        let (a, b) = (x.min(y), x.max(y));
        if self.units[a].lang != self.units[b].lang {
            self.tally.cross_lang_dropped += 1;
            return;
        }
        if set.insert((a, b)) {
            let key = format!("{src}/{}", self.units[a].lang);
            *self.tally.raw_by.entry(key).or_insert(0) += 1;
        }
    }

    /// Line-anchored pair (S1/S3): both lines must land inside
    /// admitted units or the pair is ledgered, never guessed.
    fn push_lines(&mut self, set: &mut BTreeSet<(usize, usize)>, ab: [(&str, i64); 2], src: &str) {
        match (self.owner(ab[0].0, ab[0].1), self.owner(ab[1].0, ab[1].1)) {
            (Some(x), Some(y)) => self.push(set, x, y, src),
            _ => self.tally.unowned_dropped += 1,
        }
    }

    /// S3: raw fingerprint co-occurrence, NO extension — wider than
    /// any reasonable candidate pass on purpose (§4.2), walking the
    /// SAME hash grouping as the T1/T2 extension pass (one throat).
    fn fingerprint_pairs(&mut self, instances: &[Instance]) -> BTreeSet<(usize, usize)> {
        let mut out = BTreeSet::new();
        pairs::each_hash_pair(instances, |ev| match ev {
            GroupEvent::Chained => self.tally.s3_hot_chained += 1,
            GroupEvent::Pair(a, b) => self.push_lines(
                &mut out,
                [
                    (&a.file, a.start_line as i64),
                    (&b.file, b.start_line as i64),
                ],
                "s3",
            ),
        });
        out
    }

    /// S1: the verified near-miss runs (25 <= len < 50) the T1/T2
    /// report threshold drops — reclaimed via pairs.rs's second sink.
    /// READ-ONLY (M5-close review HIGH-1): the candidate pass must
    /// never write the index — its load_streams predecessor silently
    /// re-hashed mid-run edits and orphaned their cascade-dropped
    /// edges; read_streams makes no claim on drifted files instead.
    fn near_pairs(
        &mut self,
        root: &Path,
        instances: &[Instance],
    ) -> Result<BTreeSet<(usize, usize)>> {
        let p = Params::default();
        let files = pairs::candidate_files(instances);
        let streams = walkidx::read_streams(root, &files);
        let filter = pairs::Filter {
            min_tokens: p.guarantee(),
            min_distinct: 0,
        };
        let (_, near) = pairs::clone_blocks_near(instances, &streams, filter, p.kgram);
        let mut out = BTreeSet::new();
        for run in &near {
            self.push_lines(
                &mut out,
                [
                    (&run.a_file, run.a_start as i64),
                    (&run.b_file, run.b_start as i64),
                ],
                "s1",
            );
        }
        Ok(out)
    }

    /// S4: MinHash/LSH over the structural shingle sets. Kept only as
    /// a supplementary source (F22) — the band-group distribution and
    /// chain count are published so its discriminative power is a
    /// number, not a hope.
    fn structural_pairs(&mut self) -> BTreeSet<(usize, usize)> {
        let mut buckets: BTreeMap<(usize, u64), Vec<usize>> = BTreeMap::new();
        for (id, u) in self.units.iter().enumerate() {
            assert!(
                !u.sig.is_empty(),
                "{}: admitted unit with empty shingles",
                u.path
            );
            let sig = minhash::signature(&u.sig, LSH_SHAPE.0);
            for key in minhash::band_keys(&sig, LSH_SHAPE.1, LSH_SHAPE.2) {
                buckets.entry(key).or_default().push(id);
            }
        }
        let mut out = BTreeSet::new();
        for ids in buckets.values().filter(|ids| ids.len() > 1) {
            self.band_group(&mut out, ids);
        }
        out
    }

    /// Emit one S4 band group (chained above the cap, tallied).
    fn band_group(&mut self, out: &mut BTreeSet<(usize, usize)>, ids: &[usize]) {
        *self
            .tally
            .s4_band_groups
            .entry(ids.len() as u64)
            .or_insert(0) += 1;
        if ids.len() > HOT_GROUP_CAP {
            self.tally.s4_hot_chained += 1;
            for w in ids.windows(2) {
                self.push(out, w[0], w[1], "s4");
            }
        } else {
            for (i, &a) in ids.iter().enumerate() {
                for &b in &ids[i + 1..] {
                    self.push(out, a, b, "s4");
                }
            }
        }
    }
}
