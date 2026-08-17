//! TS/TSX rungs (design §4 row 1). R1 relative + extension order —
//! the order IS the norm, so multiple hits are not ambiguous; R2 the
//! ESM `.js` → `.ts` rewrite, only when the TS twin exists and the
//! JS twin does not; R3 nearest tsconfig baseUrl+paths (extends
//! chain ≤ 8 with cycle check, else config_depth; two distinct paths
//! hits are ambiguous_paths); R4 workspace member by name/exports
//! subpath — membership is derived from the in-scope package.json
//! set (both the file set and config bytes sit in resolve_key, so
//! membership can never go stale), duplicate names are ambiguous;
//! R5 bare specifier declared in dependencies or present under
//! node_modules/ ⇒ External. Everything else is Unresolved — a
//! relative spec with no in-scope target is out_of_scope, and a
//! unique member whose export target is not a lang file terminates
//! there too (falling to R5 would call an in-corpus package
//! External).

use super::{Outcome, Reason, Scope};
use crate::graph::roots::{self, TsChain};
use serde_json::Value;
use std::collections::BTreeSet;

/// Extension order — normative, first hit wins (design §4).
const EXTS: [&str; 5] = ["ts", "tsx", "d.ts", "mts", "cts"];

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let dir = from.rfind('/').map_or("", |i| &from[..i]);
    if spec.starts_with("./") || spec.starts_with("../") {
        if let Some(path) = relative(dir, spec, scope.files) {
            return Outcome::Resolved { path, rung: 1 };
        }
        if let Some(path) = esm_rewrite(dir, spec, scope) {
            return Outcome::Resolved { path, rung: 2 };
        }
        return Outcome::Unresolved(Reason::OutOfScope);
    }
    match tsconfig_rung(dir, spec, scope) {
        Ok(Some(outcome)) => return outcome,
        Ok(None) => {}
        Err(reason) => return Outcome::Unresolved(reason),
    }
    match workspace_rung(spec, scope) {
        Some(outcome) => outcome,
        None => bare_rung(dir, spec, scope),
    }
}

/// The candidate sequence for one extensionless base: exact file,
/// then base.{ext}, then base/index.{ext} — first in-scope hit wins.
fn first_hit(base: &str, files: &BTreeSet<String>) -> Option<String> {
    if files.contains(base) {
        return Some(base.to_string());
    }
    for ext in EXTS {
        let cand = format!("{base}.{ext}");
        if files.contains(&cand) {
            return Some(cand);
        }
    }
    for ext in EXTS {
        let cand = format!("{base}/index.{ext}");
        if files.contains(&cand) {
            return Some(cand);
        }
    }
    None
}

/// R1: join + normalize, walk the extension order.
fn relative(dir: &str, spec: &str, files: &BTreeSet<String>) -> Option<String> {
    first_hit(&roots::join_rel(dir, spec)?, files)
}

/// R2: `./x.js` → `x.ts` (and .mjs/.cjs → .mts/.cts), only when the
/// TS twin is in scope AND the JS twin is absent on disk — a real JS
/// artifact means the import is not a rewritten TS reference.
fn esm_rewrite(dir: &str, spec: &str, scope: &Scope) -> Option<String> {
    let (stem, ts_ext) = [("js", "ts"), ("mjs", "mts"), ("cjs", "cts")]
        .iter()
        .find_map(|(js, ts)| Some((spec.strip_suffix(&format!(".{js}"))?, *ts)))?;
    let base = roots::join_rel(dir, stem)?;
    let target = format!("{base}.{ts_ext}");
    let js_twin = roots::join_rel(dir, spec)?;
    (scope.files.contains(&target) && !scope.root.join(js_twin).is_file()).then_some(target)
}

/// R3: nearest tsconfig paths (all matching pattern/target pairs
/// collected; two distinct in-scope hits ⇒ ambiguous_paths), then
/// the baseUrl join for bare specifiers.
fn tsconfig_rung(dir: &str, spec: &str, scope: &Scope) -> Result<Option<Outcome>, Reason> {
    let opts = match roots::ts_options(scope.root, dir) {
        TsChain::None => return Ok(None),
        TsChain::Broken => return Err(Reason::ConfigDepth),
        TsChain::Ok(opts) => opts,
    };
    let mut cands = Vec::new();
    for (pattern, targets) in &opts.paths {
        let Some(captured) = match_pattern(pattern, spec) else {
            continue;
        };
        for target in targets {
            cands.push((opts.paths_anchor.clone(), target.replacen('*', captured, 1)));
        }
    }
    let hits = distinct_hits(&cands, scope.files);
    match hits.len() {
        1 => {
            let path = hits.into_iter().next().expect("len checked");
            return Ok(Some(Outcome::Resolved { path, rung: 3 }));
        }
        n if n > 1 => return Err(Reason::AmbiguousPaths),
        _ => {}
    }
    if let Some(base_dir) = &opts.base_dir
        && let Some(base) = roots::join_rel(base_dir, spec)
        && let Some(path) = first_hit(&base, scope.files)
    {
        return Ok(Some(Outcome::Resolved { path, rung: 3 }));
    }
    Ok(None)
}

/// The shared "collect distinct in-scope hits" throat: each (anchor,
/// target) candidate goes through join + the extension order; the
/// caller judges the count (1 = resolved, more = its own ambiguity
/// reason).
fn distinct_hits(cands: &[(String, String)], files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut hits = BTreeSet::new();
    for (anchor, target) in cands {
        if let Some(base) = roots::join_rel(anchor, target)
            && let Some(hit) = first_hit(&base, files)
        {
            hits.insert(hit);
        }
    }
    hits
}

/// Single-`*` glob match; returns the captured segment ("" for an
/// exact pattern).
fn match_pattern<'a>(pattern: &str, spec: &'a str) -> Option<&'a str> {
    match pattern.split_once('*') {
        None => (pattern == spec).then_some(""),
        Some((pre, post)) => spec
            .strip_prefix(pre)
            .and_then(|rest| rest.strip_suffix(post)),
    }
}

/// R4: in-scope workspace member by package name; the subpath goes
/// through the member's exports map. None = not a member, fall on.
fn workspace_rung(spec: &str, scope: &Scope) -> Option<Outcome> {
    let (name, subpath) = split_bare(spec);
    let members = super::members(scope, "package.json", roots::package, |p| {
        p.name.as_deref() == Some(name)
    });
    match members.len() {
        0 => None,
        1 => Some(member_target(&members[0], subpath, scope)),
        _ => Some(Outcome::Unresolved(Reason::AmbiguousWorkspace)),
    }
}

/// Resolve a subpath through one member's exports: every string leaf
/// of the subpath entry is a candidate; exactly one distinct
/// in-scope hit resolves, several are ambiguous, none terminates as
/// out_of_scope (module doc: never fall through to R5 here).
fn member_target(member: &roots::Package, subpath: &str, scope: &Scope) -> Outcome {
    let mut leaves = Vec::new();
    if let Some(entry) = member
        .exports
        .as_ref()
        .and_then(|e| exports_entry(e, subpath))
    {
        string_leaves(entry, &mut leaves);
    }
    let cands: Vec<(String, String)> = leaves
        .into_iter()
        .map(|leaf| (member.dir.clone(), leaf))
        .collect();
    let hits = distinct_hits(&cands, scope.files);
    match hits.len() {
        1 => Outcome::Resolved {
            path: hits.into_iter().next().expect("len checked"),
            rung: 4,
        },
        0 => Outcome::Unresolved(Reason::OutOfScope),
        _ => Outcome::Unresolved(Reason::AmbiguousExports),
    }
}

/// The exports entry for a subpath: a top-level map keyed by "./…"
/// selects the entry; a bare (non-map or condition-only) exports
/// value IS the "." entry.
fn exports_entry<'a>(exports: &'a Value, subpath: &str) -> Option<&'a Value> {
    let key = if subpath.is_empty() {
        ".".to_string()
    } else {
        format!("./{subpath}")
    };
    match exports {
        Value::Object(map) if map.keys().any(|k| k.starts_with('.')) => map.get(&key),
        other => (key == ".").then_some(other),
    }
}

/// All string leaves under an exports entry (conditions nest).
fn string_leaves(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Object(map) => {
            for v in map.values() {
                string_leaves(v, out);
            }
        }
        _ => {}
    }
}

/// R5: bare specifier declared in the nearest package.json deps, or
/// physically present under the root node_modules/ ⇒ External.
/// Both facts ride the sweep memo — the walk-up parse and the fs
/// stat used to repeat per SITE (review MED class).
fn bare_rung(dir: &str, spec: &str, scope: &Scope) -> Outcome {
    let (name, _) = split_bare(spec);
    let declared = scope
        .memo
        .cached("ts_near", dir, || roots::nearest_package(scope.root, dir))
        .as_ref()
        .as_ref()
        .is_some_and(|p| p.deps.iter().any(|d| d == name));
    let vendored = *scope.memo.cached("ts_nm", name, || {
        scope.root.join("node_modules").join(name).is_dir()
    });
    if declared || vendored {
        return Outcome::External { rung: 5 };
    }
    Outcome::Unresolved(Reason::OutOfScope)
}

/// Package name vs subpath ("@scope/name/sub" → "@scope/name", "sub").
fn split_bare(spec: &str) -> (&str, &str) {
    let segments = if spec.starts_with('@') { 2 } else { 1 };
    let mut idx = 0;
    for _ in 0..segments {
        match spec[idx..].find('/') {
            Some(i) => idx += i + 1,
            None => return (spec, ""),
        }
    }
    (&spec[..idx - 1], &spec[idx..])
}
