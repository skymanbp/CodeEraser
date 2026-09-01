//! The Haskell LangSpec table (M5-3k). Lives outside spec.rs per the
//! design's split plan (RM16: spec.rs is a 300-line throat).
//!
//! Every kind below is AST-probed against tree-sitter-haskell 0.23.1
//! (probe transcripts 2026-08-14, two rounds: constructs + fields),
//! never guessed. Haskell has NO external cognitive-complexity
//! oracle — S3776 names no Haskell constructs — so each mapping is a
//! recorded STANCE with its nearest whitepaper analogue; the full
//! divergence register lives in contracts/coc-haskell-divergences.md
//! and every entry is pinned by tests/sonar_whitepaper.rs.
//!
//! Key probe facts the table stands on:
//! - one `function` node PER EQUATION (same name, own patterns);
//!   with_nth keys the equations apart, same as Rust impl siblings
//! - `bind` is three things at once: a top-level/where/let value
//!   binding (HAS a `name` field), a do-statement `x <- act`, and a
//!   pattern bind (both name-less) — hence fn_named_only_kinds
//! - `infix` carries fields left_operand/operator/right_operand (the
//!   `operator` field is what cyclo/cognitive match on; the operand
//!   alias is handled in cognitive.rs)
//! - guard alternatives are `guards` nodes; each condition inside is
//!   a `boolean` / `pattern_guard` / `let` qualifier
//! - if-then-else is the `conditional` EXPRESSION — the ternary
//!   analogue, so else-if chains pay nesting like nested ternaries
//!   (S3776's else-if exemption covers statement ifs)

use super::spec::{LangSpec, NameStyle};

pub static HASKELL: LangSpec = LangSpec {
    // Every named binding is a unit — equations (`function`) and
    // value binds (`bind`, name-gated). Local where/let binds are
    // standalone units, the Rust-closure precedent; a 60-line
    // point-free pipeline must not escape the E01 function gate.
    fn_kinds: &["function", "bind"],
    param_list_kinds: &["patterns"],
    cc_kinds: &[
        "conditional",
        // every case/lambda_case arm, wildcard included (Rust
        // match_arm precedent: case is total, `_` is a real path;
        // divergence register: gocyclo's default-skip not adopted)
        "alternative",
        // one per guard CONDITION, not per `guards` group: `| a, b`
        // is two decisions, matching `&&` counting; also the list-
        // comprehension filter qualifier (Python if_clause precedent)
        "boolean",
        "pattern_guard",
        // list-comprehension generator (Python for_in_clause
        // precedent: a real branch path CC counts, CoC does not)
        "generator",
    ],
    // && and || arrive as `operator` leaves under `infix`; matched by
    // TEXT via the operator field, same as every other language.
    cc_operators: &["&&", "||"],
    chain_kinds: &[],
    coc_nesting_kinds: &["conditional", "case", "multi_way_if"],
    // each guarded alternative is +1 flat — the elif analogue
    // (whitepaper p.7 hybrid increments: no nesting penalty)
    coc_flat_kinds: &["guards"],
    // lambdas absorb into their host and raise nesting only
    // (whitepaper p.13 nesting-level list; Go func_literal precedent)
    coc_nest_only_kinds: &["lambda", "lambda_case"],
    coc_operators: &["&&", "||"],
    // Haskell has no labeled jumps.
    coc_jump_kinds: &[],
    label_kinds: &[],
    // haddock (`-- |`, `{-| -}`) is a distinct kind beside comment.
    comment_kinds: &["comment", "haddock"],
    name_style: NameStyle::MixedCaps,
    // string literals lex as ONE `string` leaf (quotes inside the
    // token) — no anonymous delimiter tokens exist to classify.
    literal_delims: &[],
    // BOTH unit kinds are name-gated: `bind` is also the do-statement
    // / pattern-bind kind, and `function` is ALSO the arrow TYPE
    // (`A -> B` inside a signature — battery-caught: a 3-arg
    // signature minted three spurious units). Value-level equations
    // and binds always carry the `name` field; the impostors never do.
    fn_named_only_kinds: &["function", "bind"],
    // `f a b` nests as apply(apply(f, a), b), so only the innermost
    // application carries a `variable` in callee position: one
    // application chain yields one edge, not one per argument.
    call_kinds: &["apply"],
    call_name_kinds: &["variable"],
    // No receiver syntax: a Haskell equation is not a method.
    call_member_kinds: &[],
    call_self_words: &[],
    // a class method is a top-level name in Haskell, bare-callable
    // like any other; the probe finds no member scope to exclude
    call_member_scopes: &[],
};
