//! The joins the C-family rungs share (plan v2.30 step 5b, item 14):
//! what one searched place yields for a name, the first non-empty
//! step along an invocation's chain, a forced include's target, and
//! the targets one `#include` reaches under one chain. The per-site
//! rung (c.rs) and the include closure that attributes headers to the
//! translation units compiling them (c_index.rs) both walk this one
//! search order; the deadcode request reads the forced targets through
//! it too, with no Scope in hand — so every join here takes the walked
//! file set, never the filesystem.

use super::{Scope, paths};
use crate::graph::compdb;
use crate::graph::compdb_flags::{Chain, Search};
use crate::graph::roots;
use std::collections::BTreeSet;
use std::path::Path;

/// The name and form of an include spec: `<x>` is the system form.
pub(super) fn form(spec: &str) -> (&str, bool) {
    match spec.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        Some(inner) => (inner, true),
        None => (spec, false),
    }
}

/// The walked file a directory holds under a name.
pub(super) fn in_scope(dir: &str, name: &str, files: &BTreeSet<String>) -> Option<String> {
    roots::join_rel(dir, name).filter(|p| files.contains(p))
}

/// The walked file one searched place yields for a name: a directory
/// joins it; a framework directory (`-F`, `-iframework`) answers
/// `A/B.h` as `A.framework/Headers/B.h`, then `PrivateHeaders`
/// (clang HeaderSearch::DoFrameworkLookup), and a name with no `/` is
/// no framework header. Whether a versioned bundle's `Headers` alias
/// is a walked path is the walk's own symlink stance, not this join's.
pub(super) fn under(search: &Search, name: &str, files: &BTreeSet<String>) -> Option<String> {
    match search {
        Search::Dir(dir) => in_scope(dir, name, files),
        Search::Framework(dir) => {
            let (fw, rest) = name.split_once('/')?;
            ["Headers", "PrivateHeaders"]
                .iter()
                .find_map(|sub| in_scope(dir, &format!("{fw}.framework/{sub}/{rest}"), files))
        }
    }
}

/// The hits of the first non-empty step along one chain for a form:
/// the quoted form asks the include stack's directories (cl's second
/// step — "the directories of the currently opened include files";
/// empty outside the MSVC dialect) then the `-iquote` class; both
/// forms then ask the `-I` class and the system class, first hit. Two
/// stack directories holding the name are two hits — the register's
/// ambiguity, since two include paths opened them.
pub(super) fn along(
    chain: &Chain,
    system: bool,
    name: &str,
    stack: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> BTreeSet<String> {
    if !system {
        let stacked: BTreeSet<String> = stack
            .iter()
            .filter_map(|d| in_scope(d, name, files))
            .collect();
        if !stacked.is_empty() {
            return stacked;
        }
        if let Some(hit) = chain.quote.iter().find_map(|s| under(s, name, files)) {
            return [hit].into();
        }
    }
    chain
        .bracket
        .iter()
        .chain(&chain.system)
        .find_map(|s| under(s, name, files))
        .into_iter()
        .collect()
}

/// A forced include's target (`-include x.h`, `-imacros`, `/FI`): an
/// absolute path placed into the tree; else "the preprocessor's
/// working directory instead of the directory containing the main
/// source file" (GCC, `-include`; clang resolves the predefines
/// buffer's includes "from the current working directory instead of
/// relative to the main file", PPDirectives.cpp) — the entry's
/// directory — then the quoted chain. An entry whose directory lies
/// outside the tree skips that step.
pub(super) fn forced(
    dir: Option<&str>,
    spec: &str,
    chain: &Chain,
    root: &Path,
    files: &BTreeSet<String>,
) -> Option<String> {
    if compdb::is_absolute(spec) {
        return compdb::relativize(&compdb::root_text(root), "", spec)
            .filter(|p| files.contains(p));
    }
    dir.and_then(|d| in_scope(d, spec, files))
        .or_else(|| along(chain, false, spec, &BTreeSet::new(), files).pop_first())
}

/// Every target one `#include` of `from` reaches under one chain — the
/// closure's step: the file's own directory (quoted form, unless the
/// chain's `-I-` inhibits it), the declared roots, then the chain; the
/// first step with a hit answers, with all of its hits.
pub(super) fn targets(
    from: &str,
    spec: &str,
    chain: &Chain,
    stack: &BTreeSet<String>,
    scope: &Scope,
) -> BTreeSet<String> {
    let (name, system) = form(spec);
    if name.is_empty() {
        return BTreeSet::new();
    }
    if !system
        && chain.own_dir
        && let Some(p) = in_scope(&roots::parent_dir(from), name, scope.files)
    {
        return [p].into();
    }
    let declared = paths::declared("c", name, scope);
    if !declared.is_empty() {
        return declared;
    }
    along(chain, system, name, stack, scope.files)
}
