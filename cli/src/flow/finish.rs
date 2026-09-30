//! The file-wide noreturn names and the numbering pass (plan v2.31
//! step 4 A2; §5.1 rule 3): the attribute scan that fixes which
//! callees never return before any unit lowers, and the pre-order
//! walk that turns a finished tree into the four integer tables.

use super::build::{DYNAMIC, Lowerer};
use super::lower::{Legend, Unit};
use super::spec::FlowSpec;
use std::collections::HashSet;
use tree_sitter::Node;

/// The names a file declares noreturn through an attribute (§5.1 rule
/// 3: `_Noreturn`, `[[noreturn]]`, `__attribute__((noreturn))`): the
/// declaration holding a node of the row's kind whose text names the
/// word, spelled by its declarator chain's leaf and that leaf's last
/// `::` segment.
pub(super) fn noreturn_in_file(root: Node<'_>, src: &[u8], flow: &FlowSpec) -> HashSet<String> {
    let mut out = HashSet::new();
    if flow.noreturn_attrs.is_empty() {
        return out;
    }
    for node in crate::scan::ast::preorder(root, |_| true, crate::scan::ast::children) {
        let text = node.utf8_text(src).unwrap_or("");
        let hit = flow
            .noreturn_attrs
            .iter()
            .any(|(kind, word)| node.kind() == kind && text.contains(word.as_str()));
        let holder = crate::scan::ast::ancestors(node)
            .find(|a| a.child_by_field_name("declarator").is_some());
        if let (true, Some(holder)) = (hit, holder)
            && let Some((leaf, _)) = crate::scan::declarator::chain(holder)
        {
            let name = leaf.utf8_text(src).unwrap_or("").to_owned();
            out.insert(name.rsplit("::").next().unwrap_or("").to_owned());
            out.insert(name);
        }
    }
    out
}

/// The tree numbered in pre-order and read out into rows and legend.
pub(super) fn finish(
    l: Lowerer<'_>,
    nth: usize,
    name: String,
    start: u32,
    end: u32,
) -> Result<Unit, String> {
    let mut order = Vec::new();
    let mut stack: Vec<usize> = l.roots.iter().rev().copied().collect();
    while let Some(n) = stack.pop() {
        order.push(n);
        stack.extend(l.nodes[n].children.iter().rev());
    }
    let mut seq = vec![-1i64; l.nodes.len()];
    for (s, &n) in order.iter().enumerate() {
        seq[n] = s as i64;
    }
    let mut unit = Unit {
        nth,
        name,
        start_line: start,
        end_line: end,
        params: 0,
        dynamic: false,
        stmts: Vec::new(),
        vars: Vec::new(),
        uses: Vec::new(),
        legend: Legend {
            stmt_at: Vec::new(),
            stmt_text: Vec::new(),
            var_name: Vec::new(),
            var_at: Vec::new(),
        },
    };
    push_stmts(&l, &order, &seq, &mut unit)?;
    push_vars(&l, &seq, &mut unit)?;
    Ok(unit)
}

/// The statement and access rows in pre-order; a jump whose target
/// never entered the tree refuses the unit.
fn push_stmts(
    l: &Lowerer<'_>,
    order: &[usize],
    seq: &[i64],
    unit: &mut Unit,
) -> Result<(), String> {
    for &n in order {
        let t = &l.nodes[n];
        let aux = match t.aux {
            Some(a) if seq[a] >= 0 => seq[a],
            Some(_) => return Err(format!("a jump at {}:{} lost its target", t.at.0, t.at.1)),
            None => 0,
        };
        let parent = t.parent.map_or(-1, |p| seq[p]);
        unit.stmts.push([seq[n], parent, t.kind, t.flags, aux]);
        unit.dynamic |= t.flags >> DYNAMIC & 1 == 1;
        unit.legend.stmt_at.push(t.at);
        unit.legend.stmt_text.push(t.text.clone());
        unit.uses
            .extend(t.uses.iter().map(|&(v, m)| [seq[n], v as i64, m]));
    }
    Ok(())
}

/// The variable rows; a declaration at a statement outside the tree
/// refuses the unit.
fn push_vars(l: &Lowerer<'_>, seq: &[i64], unit: &mut Unit) -> Result<(), String> {
    for (v, var) in l.vars.iter().enumerate() {
        let decl = var.decl.map_or(-1, |d| seq[d]);
        if var.decl.is_some() && decl < 0 {
            return Err(format!(
                "`{}` is declared at a statement outside the tree",
                var.name
            ));
        }
        unit.params += (var.flags & 1) as u32;
        unit.vars.push([v as i64, decl, var.flags]);
        unit.legend.var_name.push(var.name.clone());
        unit.legend.var_at.push(var.at);
    }
    Ok(())
}
