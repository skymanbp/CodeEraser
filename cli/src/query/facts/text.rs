//! The name-shaped tables: `mention` off the mention pass's own
//! table (the identifier hashes it stores are the hashes a program's
//! `"name"` lexes to), `class` off the `[[rules.class]]` matcher the
//! scan uses, `set` as each glob the program spelled expanded over
//! the file-shaped nodes through the exclude list's own dialect.

use super::graph::Nodes;
use super::{Ctx, Labels, Sink};
use crate::query::legend;
use anyhow::{Result, anyhow};

pub fn fill(ctx: &Ctx<'_>, sets: &[String], sink: &mut Sink, labels: &mut Labels) -> Result<()> {
    if sink.wants("mention") {
        mentions(ctx, sink)?;
    }
    if sink.wants("class") {
        classes(ctx, sink, labels)?;
    }
    if sink.wants("set") {
        for (id, glob) in sets.iter().enumerate() {
            let set = crate::scan::globs::compile_inclusions(
                ctx.root,
                std::slice::from_ref(glob),
                "query glob",
            )
            .map_err(|e| anyhow!("{e}"))?;
            for (i, kind) in ctx.nodes.kinds.iter().enumerate() {
                if Nodes::file_shaped(*kind)
                    && crate::scan::globs::selected(&set, &ctx.nodes.paths[i])
                {
                    sink.row("set", vec![id as u64, i as u64]);
                }
            }
        }
    }
    Ok(())
}

/// The mention pass refreshed on the index, then its rows: every
/// (identifier hash, file) of a file that is a node here.
fn mentions(ctx: &Ctx<'_>, sink: &mut Sink) -> Result<()> {
    crate::mention::refresh(ctx.root, ctx.idx)?;
    let rows: Vec<(String, i64)> = crate::graph::load::rows(
        ctx.idx.raw(),
        "SELECT f.path, m.ident_hash FROM mentions m
         JOIN mention_files f ON f.id = m.file_id ORDER BY f.path, m.ident_hash",
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    for (path, hash) in rows {
        if let Some(node) = ctx.nodes.of_path(&path) {
            sink.row("mention", vec![hash as u64, node]);
        }
    }
    Ok(())
}

/// Each file node's declared class, by the class's name.
fn classes(ctx: &Ctx<'_>, sink: &mut Sink, labels: &mut Labels) -> Result<()> {
    let cfg = crate::config::Config::load(ctx.root).map_err(|e| anyhow!("{e}"))?;
    let classes =
        crate::scan::classes::Classes::compile(ctx.root, &cfg.rules).map_err(|e| anyhow!("{e}"))?;
    for (i, kind) in ctx.nodes.kinds.iter().enumerate() {
        if *kind != 0 {
            continue;
        }
        let c = classes.class_of(&ctx.nodes.paths[i]);
        let Some(class) = usize::try_from(c)
            .ok()
            .filter(|c| *c > 0)
            .and_then(|c| cfg.rules.class.get(c - 1))
        else {
            continue;
        };
        labels
            .names
            .entry(legend::sym(&class.name))
            .or_insert_with(|| class.name.clone());
        sink.row("class", vec![i as u64, legend::sym(&class.name)]);
    }
    Ok(())
}
