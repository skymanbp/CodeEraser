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
mod tests {
    use super::mention_name;

    /// One case per line: `path key ⇒ name`, or `⇒ -` for out of
    /// domain — K24's extraction half (the frozen out-of-domain
    /// spellings of §3.1/§2) beside the shapes that stay in. The keys
    /// are the producer's own spellings (`(anonymous)/0` is what a TS
    /// anonymous default export is keyed); the producer-measured
    /// witnesses of the same exits are in mention::conv::tests.
    #[test]
    fn the_domain_is_exactly_the_single_token_names() {
        for case in [
            "a.rs open/0 ⇒ open",
            "a.rs CLASSES ⇒ CLASSES",
            "a.rs cellar ⇒ cellar",
            "a.rs impl A ⇒ -",
            "a.rs impl Show for A ⇒ -",
            "a.ts (anonymous)/0 ⇒ -",
            "a.rs r#type/0 ⇒ -",
            "a.rs a/b/2 ⇒ -",
            "a.py x/ ⇒ -",
            "a.go (T) add/1 ⇒ add",
            "a.go (*pkg.Cache[K, V]) M/0 ⇒ M",
            "a.go free/1 ⇒ free",
            "a.py __init__/1 ⇒ -",
            "a.py __main/0 ⇒ __main",
            "a.py public_call/0 ⇒ public_call",
            "a.ts $ZodString/0 ⇒ $ZodString",
            "a.rs $foo/0 ⇒ -",
            "a.ts \"~validate\"/0 ⇒ -",
            "a.ts \"zod 3\"/0 ⇒ -",
            "a.ts 图_report/0 ⇒ -",
            "a.hs foo'/1 ⇒ -",
            "a.hs unbox#/1 ⇒ -",
            "a.hs (<+>)/2 ⇒ -",
            "a.hs fmtRow/1 ⇒ fmtRow",
            "a.md Heading ⇒ -",
            "a.js x/0 ⇒ -",
            "a.txt x/0 ⇒ -",
        ] {
            let (input, want) = case.rsplit_once(" ⇒ ").expect("case has ` ⇒ `");
            let (rel, key) = input.split_once(' ').expect("path then key");
            let want = (want != "-").then(|| want.to_string());
            assert_eq!(mention_name(rel, key), want, "{case:?}");
        }
    }
}
