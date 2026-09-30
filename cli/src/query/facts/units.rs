//! The unit tables: the index's symbol rows in one dense order
//! (file, first line, last line, key, nth), each seated in its file
//! node; kind, span, declared name, visibility bit, parameter count
//! off the key; and — only when read — the three complexity numbers
//! the core derives for `ce scan`, matched to units by span.

use super::{Ctx, Labels, Sink};
use crate::fourclass::visibility::VIS_EXPORTED;
use crate::query::legend::{self, UNIT_KINDS};
use anyhow::Result;
use std::collections::BTreeMap;

/// One seated unit.
pub struct Unit {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub start: i64,
    pub end: i64,
}

/// The unit universe: by id, and by identity.
pub struct Units {
    pub rows: Vec<Unit>,
    pub ids: BTreeMap<(String, String, i64), u64>,
}

impl Units {
    /// The units of `path` whose span lies inside `[start, end]`, in
    /// id order — a T1/T2 block's whole-unit twins are read off this.
    pub fn covered(&self, path: &str, start: i64, end: i64) -> Vec<u64> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, u)| u.path == path && u.start >= start && u.end <= end)
            .map(|(i, _)| i as u64)
            .collect()
    }
}

/// The unit tables; the universe comes back for the pair tables.
pub fn fill(ctx: &Ctx<'_>, sink: &mut Sink, labels: &mut Labels) -> Result<Units> {
    let mut rows: Vec<crate::graph::symbols::SymbolRow> =
        crate::graph::symbols::symbol_rows(ctx.idx)?
            .into_iter()
            .filter(|s| ctx.nodes.of_path(&s.path).is_some())
            .collect();
    rows.sort_by(|a, b| {
        (&a.path, a.start_line, a.end_line, &a.key, a.nth).cmp(&(
            &b.path,
            b.start_line,
            b.end_line,
            &b.key,
            b.nth,
        ))
    });
    let mut units = Units {
        rows: Vec::new(),
        ids: BTreeMap::new(),
    };
    for (i, s) in rows.iter().enumerate() {
        let id = i as u64;
        let node = ctx
            .nodes
            .of_path(&s.path)
            .expect("filtered to seated paths");
        sink.row("unit", vec![id, node]);
        let kind = usize::try_from(s.kind - 1)
            .ok()
            .and_then(|k| UNIT_KINDS.get(k))
            .copied()
            .unwrap_or("other");
        sink.row("unit_kind", vec![id, legend::sym(kind)]);
        sink.row(
            "unit_lines",
            vec![id, (s.end_line - s.start_line + 1).max(0) as u64],
        );
        sink.row("unit_at", vec![id, s.start_line as u64]);
        if s.vis & VIS_EXPORTED != 0 {
            sink.row("exported", vec![id]);
        }
        if let Some(name) = crate::mention::name::mention_name(&s.path, &s.key) {
            labels
                .names
                .entry(legend::sym(&name))
                .or_insert_with(|| name.clone());
            sink.row("named", vec![id, legend::sym(&name)]);
        }
        if let Some(n) = s.key.rsplit('/').next().and_then(|t| t.parse::<u64>().ok()) {
            sink.row("params", vec![id, n]);
        }
        labels.units.push(format!(
            "{}:{}-{} {}",
            s.path, s.start_line, s.end_line, s.key
        ));
        units.ids.insert((s.path.clone(), s.key.clone(), s.nth), id);
        units.rows.push(Unit {
            path: s.path.clone(),
            key: s.key.clone(),
            nth: s.nth,
            start: s.start_line,
            end: s.end_line,
        });
    }
    if sink.wants_any(&["coc", "cyclo", "nesting"]) {
        complexity(ctx, &units, sink)?;
    }
    Ok(units)
}

/// The core's three complexity numbers per function (scan::settle),
/// each seated on the unit sharing its file and span — a span two
/// units share goes to the one whose key names the function, and to
/// no unit when neither does.
fn complexity(ctx: &Ctx<'_>, units: &Units, sink: &mut Sink) -> Result<()> {
    let mut by_span: BTreeMap<(&str, i64, i64), Vec<u64>> = BTreeMap::new();
    for (i, u) in units.rows.iter().enumerate() {
        by_span
            .entry((u.path.as_str(), u.start, u.end))
            .or_default()
            .push(i as u64);
    }
    let settled = crate::scan::settle(ctx.root, ctx.core)?;
    for f in &settled.files {
        for m in &f.functions {
            let key = (f.path.as_str(), m.start_line as i64, m.end_line as i64);
            let Some(seats) = by_span.get(&key) else {
                continue;
            };
            let named = |id: &u64| {
                let key = &units.rows[*id as usize].key;
                key.split('/').next() == Some(m.name.as_str())
            };
            let seat = match seats.as_slice() {
                [one] => Some(*one),
                many => many.iter().find(|id| named(id)).copied(),
            };
            let Some(id) = seat else { continue };
            sink.row("coc", vec![id, u64::from(m.cognitive)]);
            sink.row("cyclo", vec![id, u64::from(m.cyclomatic)]);
            sink.row("nesting", vec![id, u64::from(m.max_nesting)]);
        }
    }
    Ok(())
}
