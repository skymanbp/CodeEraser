//! Node's builtin module names (plan v2.30 step 5b), machine-listed:
//! `require('module').builtinModules` of Node 22.22.1 — 68 names, each
//! also importable under the `node:` prefix — plus the four modules
//! that exist under the prefix alone (`node:sea`, `node:sqlite`,
//! `node:test`, `node:test/reporters`; each `require`d on the same Node
//! to confirm, 2026-09-26). A bare specifier naming one is External at
//! the bare rung whatever any package.json declares: Node serves a
//! builtin before any node_modules lookup (the resolver's
//! LOAD_NODE_MODULES step is never reached), and a `node:` name the
//! tables lack is nothing Node can load. The two tables are the core's
//! since plan v2.32 step 2 (CE.Lang.Common.Ladder `ts`, read off
//! `tables/1`).

/// Whether `spec` names a Node builtin: the bare name from the first
/// table, or `node:` before a name from either.
pub(super) fn is_builtin(spec: &str) -> bool {
    let node = &crate::tables::get().ladder.ts;
    match spec.strip_prefix("node:") {
        Some(name) => node.builtins.contains(&name) || node.prefix_only.contains(&name),
        None => node.builtins.contains(&spec),
    }
}
