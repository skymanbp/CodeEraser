//! `mention_name` — the declaration-side name the mention veto judges
//! (sealed criterion §3.1, plan v2.17 L round piece (4)): a unit key
//! reduced to the ONE identifier the corpus would have to spell to
//! reference it, or None when no such identifier exists. None is the
//! safe answer: a declaration outside the domain is never reported
//! unmentioned, so every rejection here costs recall of the advisory
//! and never its precision.
//!
//! From a key (`name/arity`, `(T) method/arity`, `impl X`, a heading):
//!   - Markdown is out (RG9: a heading is an anchor, not an identifier);
//!   - the arity suffix goes; a Go receiver goes with `rsplit_once(") ")`
//!     — a generic receiver `(*Cache[K, V]) M` holds no `") "` inside
//!     its brackets, so the last one is always the receiver's close;
//!   - a Python dunder is out (protocol, never referenced by spelling);
//!   - what remains must be ONE token of the declaring file's own
//!     tokenizer arm (§2 invariant: `tokenize_for(ext, r) == [r]`) —
//!     `foo'`, `unbox#`, `r#type`, `(<+>)`, `"zod 3"`, `图_report`,
//!     `(anonymous)` and `impl X` all fail it, each for the reason the
//!     tokenizer itself states.
//!
//! The key's own spelling (`declared_name`, the unit key) is untouched:
//! this is a partial function FROM the key, not a second producer of it.

use super::token::{emit, whole_run_only};
use crate::scan::lang::Lang;
use std::path::Path;

/// The name the veto judges for the unit keyed `key` in the judged
/// file `rel`, or None when the declaration is out of the domain.
pub fn mention_name(rel: &str, key: &str) -> Option<String> {
    let lang = Lang::judged_path(Path::new(rel))?;
    if lang == Lang::Markdown {
        return None;
    }
    let bare = match lang {
        Lang::Go => key.rsplit_once(") ").map_or(key, |(_, method)| method),
        _ => key,
    };
    let name = de_arity(bare);
    if lang == Lang::Python && dunder(name) {
        return None;
    }
    single_token(rel, name).then(|| name.to_string())
}

/// `name/3` → `name`; a key with no all-digit tail (`CLASSES`,
/// `impl A`, a Rust `mod` name) is its own name.
fn de_arity(key: &str) -> &str {
    match key.rsplit_once('/') {
        Some((name, arity)) if !arity.is_empty() && arity.bytes().all(|b| b.is_ascii_digit()) => {
            name
        }
        _ => key,
    }
}

fn dunder(name: &str) -> bool {
    name.len() > 4 && name.starts_with("__") && name.ends_with("__")
}

/// The §2 invariant: the declaring file's arm emits exactly one token
/// from the name, and that token IS the name.
fn single_token(rel: &str, name: &str) -> bool {
    let mut emitted = 0;
    let mut whole = false;
    emit(name, whole_run_only(rel), &mut |t| {
        emitted += 1;
        whole |= t == name;
    });
    emitted == 1 && whole
}

#[cfg(test)]
#[path = "../../tests/unit/mention/name.rs"]
mod tests;
