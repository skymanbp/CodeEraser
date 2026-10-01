//! The groups merge/1 is asked about (plan v2.31 step 7, design
//! booklet §6.1; step-7 ruling 4): every T1/T2 clone family `ce dedup`
//! verified and every T3 pair `ce clone` judged, each member with its
//! tree in clone/1's postorder encoding plus merge/1's two columns.
//! A T1/T2 member is a whole unit when one admitted unit fills it (the
//! unit inside the member's lines, at least nine tenths of them — a
//! member's line span is coarser than its tokens); a family whose every
//! member is one sends those units' trees, any other sends each member's
//! maximal selected nodes under a synthetic root (a fragment), trimmed
//! to the run of top nodes every member holds (groups_trim.rs). No tree
//! carries the grammar's extras: a comment is not code a merge folds
//! (step-7 ruling 3). A group the core would refuse, or that no request
//! can carry, is never sent: it is counted by why. One member set is one
//! suggestion: a T1/T2 group covers a T3 group over the same members.

use super::groups_trim::{isomorphic, trim};
use super::slot::slot_spec;
use super::wire::TREE_NODE_CAP;
use crate::dedup::candidates::Unit;
use crate::dedup::groups::Group as Family;
use crate::dedup::t3::tree::{Extras, Top, UnitTree, file_fragments};
use crate::dedup::t3::{Outcome, build_trees};
use crate::scan::lang::Lang;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Group families — mirrors of CE.Merge.Cost.familyExact / familyNear.
pub const FAMILY_EXACT: u8 = 0;
pub const FAMILY_NEAR: u8 = 1;

/// One member: its file, its unit (`path:key#nth`; None for a
/// fragment), its 1-based line span — the clone family's, its identity
/// in `ce dedup` — the lines of the run it sends (a fragment trimmed to
/// the members' common run of tops; a whole unit's are its span), which
/// the core prices, and its tree.
pub struct Member {
    pub path: String,
    pub unit: Option<String>,
    pub lines: (usize, usize),
    pub run: (usize, usize),
    pub tree: UnitTree,
}

/// One group as sent: its family, whether its trees are fragments,
/// and its members in order.
pub struct Group {
    pub family: u8,
    pub fragment: bool,
    pub members: Vec<Member>,
}

impl Group {
    pub fn nodes(&self) -> usize {
        self.members.iter().map(|m| m.tree.lab.len()).sum()
    }

    /// The members' identities, sorted: each unit, or a fragment's
    /// `path:start-end` (its clone-family span).
    pub fn member_set(&self) -> Vec<String> {
        let at = |m: &Member| {
            m.unit
                .clone()
                .unwrap_or_else(|| format!("{}:{}-{}", m.path, m.lines.0, m.lines.1))
        };
        let mut ids: Vec<String> = self.members.iter().map(at).collect();
        ids.sort();
        ids
    }
}

/// One suggestion per member set (score::one_row_per_pair's stand, 0
/// over 1): a T1/T2 group covers a T3 group over the same members — an
/// exact isomorphism is the stronger claim. The order is kept (the sort
/// is stable and T1/T2 plans come first); the groups dropped are the
/// same suggestion twice, not unsendable, and their number is returned.
pub fn one_per_member_set(groups: &mut Vec<Group>) -> u64 {
    groups.sort_by_key(|g| g.family);
    let before = groups.len();
    let mut seen = BTreeSet::new();
    groups.retain(|g| seen.insert(g.member_set()));
    (before - groups.len()) as u64
}

/// The groups not sent, by why: T1/T2 members whose trees are not one
/// shape, a language with no slot table, a member whose tree was not
/// built (over clone/1's per-tree cap, a forest, nothing selected),
/// and a group whose nodes alone pass merge/1's request cap.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unsendable {
    pub not_isomorphic: u64,
    pub no_slot_table: u64,
    pub unbuilt: u64,
    pub over_cap: u64,
}

/// A group before its trees: the members' (path, start, end) and,
/// when every member is a whole unit, their unit ids.
struct Plan {
    family: u8,
    spans: Vec<(String, usize, usize)>,
    units: Option<Vec<usize>>,
}

/// Every group, trees built, the unsendable ones counted, one per
/// member set (the duplicates dropped counted apart).
pub fn gather(
    root: &Path,
    families: &[Family],
    units: &[Unit],
    pairs: &[(usize, usize)],
) -> Result<(Vec<Group>, Unsendable, u64)> {
    let by_path = crate::dedup::sources::by_file(units);
    let mut plans: Vec<Plan> = families.iter().map(|f| exact(f, units, &by_path)).collect();
    plans.extend(pairs.iter().map(|&(a, b)| Plan {
        family: FAMILY_NEAR,
        spans: vec![span_of(&units[a]), span_of(&units[b])],
        units: Some(vec![a, b]),
    }));
    let built = unit_trees(root, units, &plans)?;
    let frags = fragments(root, &plans)?;
    let mut out = Vec::new();
    let mut skipped = Unsendable::default();
    for p in plans {
        match settle(p, units, &built, &frags) {
            Ok(g) => out.push(g),
            Err(counter) => *counter(&mut skipped) += 1,
        }
    }
    let merged = one_per_member_set(&mut out);
    Ok((out, skipped, merged))
}

fn span_of(u: &Unit) -> (String, usize, usize) {
    (u.path.clone(), u.start_line as usize, u.end_line as usize)
}

/// A T1/T2 family's plan: its units when each member is filled by one.
fn exact(f: &Family, units: &[Unit], by_path: &BTreeMap<&str, Vec<usize>>) -> Plan {
    let whole: Option<Vec<usize>> = f
        .members
        .iter()
        .map(|m| filling(m.start, m.end, by_path.get(m.file.as_str())?, units))
        .collect();
    Plan {
        family: FAMILY_EXACT,
        spans: f
            .members
            .iter()
            .map(|m| (m.file.clone(), m.start, m.end))
            .collect(),
        units: whole,
    }
}

/// The longest unit inside [start, end] (the first of equals), kept
/// when it spans at least 90 % of the member's lines.
pub fn filling(start: usize, end: usize, ids: &[usize], units: &[Unit]) -> Option<usize> {
    let lines = |u: &Unit| (u.end_line - u.start_line + 1) as usize;
    let inside = ids
        .iter()
        .copied()
        .filter(|&i| units[i].start_line as usize >= start && units[i].end_line as usize <= end);
    let best = inside.fold(None, |best: Option<usize>, i| match best {
        Some(b) if lines(&units[b]) >= lines(&units[i]) => Some(b),
        _ => Some(i),
    })?;
    (lines(&units[best]) * 10 >= (end - start + 1) * 9).then_some(best)
}

/// The whole-unit trees every plan names, from one parse per file.
fn unit_trees(root: &Path, units: &[Unit], plans: &[Plan]) -> Result<BTreeMap<usize, Outcome>> {
    let mut ids: Vec<usize> = plans
        .iter()
        .flat_map(|p| p.units.iter().flatten().copied())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let wanted: Vec<&Unit> = ids.iter().map(|&i| &units[i]).collect();
    let built = build_trees(root, &wanted, Extras::Without)?;
    Ok(ids.into_iter().zip(built).collect())
}

type Span = (String, usize, usize);

/// A fragment member's top trees with their lines, in source order.
type Tops = Vec<Top>;

/// Every fragment plan's member top trees, one parse per file.
fn fragments(root: &Path, plans: &[Plan]) -> Result<BTreeMap<Span, Option<Tops>>> {
    let mut by_file: BTreeMap<&str, Vec<(usize, usize)>> = BTreeMap::new();
    for p in plans.iter().filter(|p| p.units.is_none()) {
        for (path, s, e) in &p.spans {
            by_file.entry(path).or_default().push((*s, *e));
        }
    }
    let mut out = BTreeMap::new();
    for (path, spans) in by_file {
        let (text, lang) = crate::dedup::walked_text(root, path)?;
        let trees = file_fragments(&text, lang, &spans, Extras::Without);
        ensure!(trees.len() == spans.len(), "{path}: one fragment per span");
        for ((s, e), t) in spans.into_iter().zip(trees) {
            out.insert((path.to_string(), s, e), t);
        }
    }
    Ok(out)
}

type Counter = fn(&mut Unsendable) -> &mut u64;

/// A plan's group, or the counter it lands in: a language without a
/// slot table first, then a tree not built, then — a fragment family
/// with no run of tops in common — a shape that is not one, then the
/// node cap, then (T1/T2 alone) a shape that is not one.
fn settle(
    p: Plan,
    units: &[Unit],
    built: &BTreeMap<usize, Outcome>,
    frags: &BTreeMap<Span, Option<Tops>>,
) -> Result<Group, Counter> {
    let tabled = |(path, _, _): &Span| {
        Lang::from_path(Path::new(path))
            .and_then(slot_spec)
            .is_some()
    };
    if !p.spans.iter().all(tabled) {
        return Err(|u| &mut u.no_slot_table);
    }
    let members = match &p.units {
        Some(ids) => ids
            .iter()
            .map(|&i| whole(&units[i], built.get(&i)?))
            .collect::<Option<Vec<Member>>>()
            .ok_or(UNBUILT)?,
        None => trimmed(&p.spans, frags)?,
    };
    let g = Group {
        family: p.family,
        fragment: p.units.is_none(),
        members,
    };
    if g.nodes() > TREE_NODE_CAP {
        return Err(|u| &mut u.over_cap);
    }
    let trees: Vec<&UnitTree> = g.members.iter().map(|m| &m.tree).collect();
    if g.family == FAMILY_EXACT && !isomorphic(&trees) {
        return Err(NOT_ISOMORPHIC);
    }
    Ok(g)
}

fn whole(u: &Unit, built: &Outcome) -> Option<Member> {
    let Outcome::Tree(t) = built else { return None };
    let lines = (u.start_line as usize, u.end_line as usize);
    Some(Member {
        path: u.path.clone(),
        unit: Some(format!("{}:{}#{}", u.path, u.key, u.nth)),
        lines,
        run: lines,
        tree: t.clone(),
    })
}

const UNBUILT: Counter = |u| &mut u.unbuilt;

/// A fragment family's members: each member's tops, trimmed to the run
/// every member holds; a member's `lines` stay its clone-family span (its
/// identity in `ce dedup`), its `run` is the kept run's lines (what the
/// core prices), the texts come off the trimmed trees.
fn trimmed(spans: &[Span], frags: &BTreeMap<Span, Option<Tops>>) -> Result<Vec<Member>, Counter> {
    let tops: Vec<&[Top]> = spans
        .iter()
        .map(|s| frags.get(s)?.as_deref())
        .collect::<Option<_>>()
        .ok_or(UNBUILT)?;
    let trees = trim(&tops).ok_or(NOT_ISOMORPHIC)?;
    Ok(spans
        .iter()
        .zip(trees)
        .map(|((path, s, e), (tree, run))| Member {
            path: path.clone(),
            unit: None,
            lines: (*s, *e),
            run,
            tree,
        })
        .collect())
}

const NOT_ISOMORPHIC: Counter = |u| &mut u.not_isomorphic;

#[cfg(test)]
#[path = "../../tests/unit/merge/groups.rs"]
mod tests;
