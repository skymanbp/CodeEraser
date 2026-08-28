//! §2 of the sealed criterion: the mention tokenizer — three emitters
//! over ONE run definition, and the fold key. Every rule in this file
//! is a `MENTION_REV` input (mod.rs): changing any of them re-derives
//! every stored row, because two machines reading one commit must
//! store the same hashes.
//!
//! A run opens on a Unicode letter, `_` or `$` and continues over
//! Unicode alphanumerics, `_` and `$` — `$` sits in BOTH sets, or
//! `$ZodString` would be scanned from `Z` and every emitter would miss
//! it. The emitters differ only in what they emit from a run, never in
//! how the run is cut:
//!   (i)   the whole run;
//!   (ii)  the script split — the run's maximal identifier-side pieces
//!         (ASCII alphanumerics, `_`, `$`: `调用$graph函数` yields
//!         `$graph`, never a bare `graph`);
//!   (iii) the `$` arm — the run's maximal `$`-free pieces, for every
//!         extension OUTSIDE the JS family (`exec $ce_entry_main` in a
//!         shell script keeps `ce_entry_main`; a `.ts` file keeps only
//!         `$ZodString`, so the `$`-twins stop hiding each other).
//! A piece equal to the run is the run (already emitted), and a piece
//! that does not open a run (digit-led: `$1` → no `1`) is dropped —
//! such a name can never satisfy the declaration-side token invariant,
//! so nothing is lost. The three run independently on the run; none
//! feeds another.

use crate::scan::lang::MENTION_WHOLE_RUN_EXTS;
use std::path::Path;

fn opens(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn continues(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// The identifier side of the script split — deliberately ASCII, not
/// "Latin": `char` has no script predicate, and a Unicode reading
/// flips domain membership (`café_report` reads as one token here and
/// three there). The non-ASCII-Latin declaration name this costs is
/// out of domain on the safe side, like every mixed-script name.
fn ident_side(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// Every run of `text`, left to right, as slices of it.
pub fn runs(text: &str) -> impl Iterator<Item = &str> {
    let mut at = 0;
    std::iter::from_fn(move || {
        let (start, end) = next_run(text, at)?;
        at = end;
        Some(&text[start..end])
    })
}

fn next_run(text: &str, from: usize) -> Option<(usize, usize)> {
    let rest = &text[from..];
    let (off, first) = rest.char_indices().find(|&(_, c)| opens(c))?;
    let start = from + off;
    let body_at = start + first.len_utf8();
    let body = &text[body_at..];
    let len = body
        .char_indices()
        .find(|&(_, c)| !continues(c))
        .map_or(body.len(), |(i, _)| i);
    Some((start, body_at + len))
}

/// Feed every token of `text` to `sink`, duplicates included (the
/// store de-duplicates per file). `whole_run_only` is the JS-family
/// arm: it silences emitter (iii) alone.
pub fn emit<'t>(text: &'t str, whole_run_only: bool, sink: &mut impl FnMut(&'t str)) {
    for run in runs(text) {
        sink(run);
        pieces(run, ident_side, sink);
        if !whole_run_only && run.contains('$') {
            pieces(run, |c| c != '$', sink);
        }
    }
}

/// The maximal substrings of `run` over `keep` — emitted when they are
/// proper substrings that open a run (module doc).
fn pieces<'t>(run: &'t str, keep: impl Fn(char) -> bool, sink: &mut impl FnMut(&'t str)) {
    for piece in run.split(|c| !keep(c)).filter(|p| !p.is_empty()) {
        if piece.len() < run.len() && piece.chars().next().is_some_and(opens) {
            sink(piece);
        }
    }
}

/// The `$` arm's table lookup: extension lower-cased before the
/// lookup (`Foo.TS` is a `.ts` file), and NO extension is the union
/// arm (`Makefile`, `Dockerfile`, `.gitignore`) — both sentences are
/// `MENTION_REV` inputs, or one commit would shard differently on two
/// machines.
pub fn whole_run_only(rel: &str) -> bool {
    Path::new(rel)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|e| MENTION_WHOLE_RUN_EXTS.contains(&e.as_str()))
}

/// Second-chance key: `_`, `-` and `$` filtered, then lower-cased —
/// `$ZodString` folds to `zodstring` and can rescue a Rust
/// `zod_string`. `$` is filtered HERE on purpose (the reference probe
/// was silent on it, which would have let two machines disagree).
pub fn fold(token: &str) -> String {
    token
        .chars()
        .filter(|c| !matches!(c, '_' | '-' | '$'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// A token this long — LITERAL characters, counted before the fold
/// filters anything (`$ZodStr` is 7 and fills; its fold `zodstr` is
/// not re-judged) — also stores its fold key. `fold`, this and
/// `segments` are public for the K23 instrument (tests/it/eval_support/
/// mention.rs), which must ask the veto's own fold channel.
pub const FOLD_MIN_CHARS: usize = 7;

/// The declaration-side segment count of the fold gate (§2 Q1): a
/// Rust name takes the fold's second chance only with ≥2 segments
/// (`_` and camel boundaries; an all-caps run is one segment, so
/// `HTTPServer` is two and `RULES` one) and ≥ FOLD_MIN_CHARS literal
/// characters. A `MENTION_REV` input like the fold itself.
pub fn segments(name: &str) -> usize {
    let mut count = 0;
    for part in name.split('_').filter(|p| !p.is_empty()) {
        let chars: Vec<char> = part.chars().collect();
        count += 1;
        for i in 1..chars.len() {
            let (prev, cur) = (chars[i - 1], chars[i]);
            let rises = cur.is_uppercase() && !prev.is_uppercase();
            let caps_end = cur.is_uppercase()
                && prev.is_uppercase()
                && chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if rises || caps_end {
                count += 1;
            }
        }
    }
    count
}

/// The bundler de-duplication witness (K41): a run shaped `\w+$\d+`
/// (`name$1`, rollup/esbuild deconflict suffixes) inside a `dist/`
/// JavaScript file. The JS arm keeps such a run whole, so its base
/// name is never emitted; counting the shape makes that cost visible.
pub fn dedup_suffixed(run: &str) -> bool {
    run.rsplit_once('$').is_some_and(|(base, digits)| {
        !base.is_empty() && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
    })
}
