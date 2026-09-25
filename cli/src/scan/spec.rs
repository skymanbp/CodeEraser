//! Per-language node-kind tables driving function extraction and both
//! complexity metrics. One data table per language instead of one code
//! path per language (the anti-fuck-u-code design: their regex control
//! flow signatures broke on every non-C-like syntax).
//!
//! Alignment targets (plan §6 M1 acceptance): lizard (TS/Py),
//! rust-code-analysis (Rust), gocyclo/gocognit (Go). Known divergences
//! are recorded in contracts/ during the M1 cross-check, not hidden.
//!
//! This file is the CONTRACT — the struct, the dispatch and the empty
//! table. The tables live beside it, one file per language family
//! (spec_launch.rs holds the M1 launch set, spec_hs.rs, spec_c.rs,
//! spec_java.rs, spec_lua.rs and spec_r.rs the later ones): a table is
//! data a reader compares against its grammar, and the contract read
//! past the 300-line line once plan v2.30 step 3 added the mechanisms
//! Java needs (RM16).

use super::lang::Lang;

/// Function-name convention for the readability naming check (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameStyle {
    /// snake_case — PEP 8 (Python), RFC 430 (Rust): no uppercase.
    Snake,
    /// mixedCaps / PascalCase — Effective Go, common TS: no underscores.
    MixedCaps,
    /// No convention enforced (Markdown).
    Any,
}

/// A list of grammar spellings — node kinds, operator tokens,
/// string delimiters or keywords, depending on the field. Named
/// because the table repeats it for nearly every entry.
pub type Kinds = &'static [&'static str];

/// How overload resolution (LangSpec::overloads) counts. A unit's
/// parameter list bounds the arguments a call may pass: every named
/// parameter child counts toward both bounds except an `optional` one
/// (a C++ default argument: the upper bound only), a `variadic` one (no
/// upper bound — C++'s C-style `...` is an anonymous token and is
/// matched by kind like the named pack declaration) and an `ignored`
/// one (Java's receiver parameter `Foo this`, which no argument fills).
/// A call passes its argument list's entries, unless one is a `spread`
/// (a C++ pack expansion `f(a...)`): then the count is unknown and
/// admits every seat. An `unreachable` unit competes for no seat at
/// all — a Java constructor is reached by `new` and `this(…)`, neither
/// of them a call kind, and a method named like its class must not
/// resolve to it.
pub struct Overloads {
    pub optional: Kinds,
    pub variadic: Kinds,
    pub ignored: Kinds,
    pub spread: Kinds,
    pub unreachable: Kinds,
}

pub struct LangSpec {
    /// Node kinds counted as standalone function units. Anything not
    /// listed here (Go func_literal, Python lambda) is absorbed into
    /// its host function by construction — no separate flag needed.
    pub fn_kinds: Kinds,
    /// fn_kinds entries that form a unit ONLY when the node carries a
    /// field, as (kind, field). Haskell's `bind` is also the do-statement
    /// `x <- act` and pattern-bind kind, and `function` also the arrow
    /// TYPE inside a signature (AST-probed 3k): the value-level equation
    /// carries `name` and the impostors never do — without the gate
    /// every do line became a unit and its complexity left the host.
    /// Java's `method_declaration` without a `body` is an abstract,
    /// interface or native signature, with nothing to measure — the C++
    /// `= default` stance (register D24); the other grammars spell a
    /// signature with a kind that never enters fn_kinds at all.
    pub fn_required_fields: &'static [(&'static str, &'static str)],
    /// Kind of the parameter-list child (direct child of a fn node).
    pub param_list_kinds: Kinds,
    /// +1 cyclomatic per node of these kinds.
    pub cc_kinds: Kinds,
    /// Binary-operator tokens adding +1 cyclomatic (short-circuit ops).
    pub cc_operators: Kinds,
    /// Kinds whose N children are joined by N-1 anonymous short-circuit
    /// tokens invisible to cc_operators (Rust let_chain: no `operator`
    /// field). CC adds N-1; CoC adds one operator run.
    pub chain_kinds: Kinds,
    /// Cognitive: structures that increment AND raise nesting. Each
    /// entry is the kind, then where its body sits: a field name or,
    /// where the grammar leaves the body unnamed (a Python except's
    /// block, a Go switch's cases), the body child's kind. Only the
    /// body raises nesting: every other child (the condition, a
    /// loop's clause, a switch's value, a catch's parameter) is the
    /// header and scores at the structure's own level. An entry that
    /// names no position nests whole (the ternaries). The positions
    /// sit in the entries, not in a table of their own: a second table
    /// lines up across the language tables as a clone of itself.
    pub coc_nesting_kinds: Kinds,
    /// Cognitive: the if kinds, EXACT — where an else branch may hang
    /// off the if's `alternative` FIELD instead of arriving as an else
    /// node (Go and Java carry the next if or the else body there
    /// directly). An if in another if's alternative is an else-if (flat
    /// +1, chain level); any other alternative that is no coc_flat node
    /// is a plain else, +1 — a Java `else return x;` included, which
    /// the block-only reading missed. A kind list, not a prefix: the
    /// ternary also has an `alternative` field (booklet §4 (d)).
    pub if_kinds: Kinds,
    /// Cognitive: flat +1 (no nesting penalty), e.g. `else`. A branch
    /// that carries a condition names its body the way a nesting
    /// entry does (Python's elif, whose condition stays at the
    /// chain's level).
    pub coc_flat_kinds: Kinds,
    /// Cognitive: raise nesting only (lambdas / inline fns).
    pub coc_nest_only_kinds: Kinds,
    /// Cognitive: operator tokens forming counted boolean runs. Split
    /// from cc_operators: the whitepaper (v1.7 p.6 "Ignore shorthand")
    /// ignores null-coalescing in CoC while CC counts it as a branch.
    pub coc_operators: Kinds,
    /// Cognitive: jump statements adding a fundamental +1 when labeled
    /// (whitepaper p.8 "Jumps to labels": goto / break L / continue L).
    pub coc_jump_kinds: Kinds,
    /// Node kind of the label child that marks a jump as labeled
    /// (AST-probed: Go `label_name`, Rust `label`, TS
    /// `statement_identifier`).
    pub label_kinds: Kinds,
    pub comment_kinds: Kinds,
    /// Readability: function-name convention (plan §4.1 naming item).
    pub name_style: NameStyle,
    /// Dedup: anonymous string-delimiter tokens that count as literal
    /// pieces. Per-language because `'` is a QUOTE only where the
    /// grammar lexes char/string content separately — in Rust `'` is
    /// the lifetime/label tick (M2 attack review: classifying it LIT
    /// made every `&'a str` signature a false clone driver).
    pub literal_delims: Kinds,
    /// Recursion (whitepaper p.8, plan v2.23): call/application node
    /// kinds. Empty = the language mints no call edges.
    pub call_kinds: Kinds,
    /// (callee field, receiver field) of every call kind. Seven grammars
    /// hang the callee off `function` and select a member INSIDE it
    /// (call_member_kinds). Java hangs the method name off `name` and
    /// the receiver off the call's own `object` field: a call carrying
    /// the receiver field takes the member road with that field as its
    /// object, one without it the bare road (booklet §4 (c)). The one
    /// spelling every reader of a call uses, with call_kinds — the arcs
    /// here, the similar advisor's callee words and the graph's call
    /// sites (graph/sites/call.rs).
    pub call_fields: (&'static str, Option<&'static str>),
    /// Callee shapes spelling a bare name, matched against the WHOLE
    /// unit name — a Go method reads `(T) g`, so `g()` never reaches it.
    pub call_name_kinds: Kinds,
    /// Callee shapes selecting a member off an object. Only the
    /// caller's own receiver counts; the name is then matched with the
    /// receiver prefix stripped.
    pub call_member_kinds: Kinds,
    /// Object spellings meaning "the thing I am a method of". Go has
    /// none: its receiver is a binding each method names itself.
    pub call_self_words: Kinds,
    /// Kinds marking a MEMBER scope — a body holding a type's members
    /// rather than a plain lexical block. Matched against a callable's
    /// container AND that container's parent, because the grammars
    /// split the distinction differently: Rust's `declaration_list` is
    /// shared by `impl`, `trait` and `mod`, so only the parent tells
    /// them apart, while TypeScript's `class_body` and `object` say it
    /// themselves. A bare name never reaches inside one — a method
    /// answers to a receiver, never to its own name alone.
    pub call_member_scopes: Kinds,
    /// Named type declarations whose `name` spells a member's OWNER
    /// (Java): the chain of these around a unit, outermost first, keys
    /// its member road the way the C family's declarator does
    /// (scan/declarator.rs). No owner when the unit's nearest member
    /// scope belongs to anything else — an anonymous class, an enum
    /// constant's body — which no call can name.
    pub owner_kinds: Kinds,
    /// Overload resolution (C++, Java): same-named callables of one
    /// scope are DIFFERENT functions, told apart by their parameters, so
    /// each is its own callable and a call reaches the one whose arity
    /// range admits its argument count — two admitting it, no edge (the
    /// undercount direction). None keeps the other reading, where
    /// same-named units of one scope are ONE callable: Haskell equations,
    /// a function written twice under two cfg / `#if` arms, a Python
    /// property's getter and setter.
    pub overloads: Option<&'static Overloads>,
    /// Kinds that BIND names locally — an import written inside a
    /// body. Names appearing under one shadow the callable enclosing
    /// it, so a bare call of that name is not provably the local one:
    /// the ignore crate's `fn symlink { use ..::symlink; symlink(..) }`
    /// was measured charging itself a recursion point. Empty where the
    /// language forbids the collision outright: Go imports are package
    /// qualified, and TypeScript and Haskell reject a local name that
    /// duplicates an import in the same scope.
    pub call_import_kinds: Kinds,
    /// (parent kind, field) pairs whose child is compile-time text the
    /// parser types as an expression all the same — a `#if` / `#elif`
    /// condition. No metric reads through one: its operators are no
    /// branches and its calls no arcs (register D1). Empty where the
    /// grammar has no preprocessor.
    pub opaque_fields: &'static [(&'static str, &'static str)],
}

pub fn spec(lang: Lang) -> &'static LangSpec {
    match lang {
        Lang::Python => &super::spec_launch::PYTHON,
        Lang::TypeScript | Lang::Tsx => &super::spec_launch::TYPESCRIPT,
        Lang::Rust => &super::spec_launch::RUST,
        Lang::Go => &super::spec_launch::GO,
        Lang::Markdown => &MARKDOWN,
        Lang::Haskell => &super::spec_hs::HASKELL,
        Lang::C => &super::spec_c::C,
        Lang::Cpp => &super::spec_c::CPP,
        Lang::Java => &super::spec_java::JAVA,
        Lang::Lua => &super::spec_lua::LUA,
        Lang::R => &super::spec_r::R,
        // The sentinel is never walked; the scan-only arm (plan
        // v2.5) is size-only like Markdown — grammar() is None for
        // all of them, so measure_file never reaches these tables:
        // the empty MARKDOWN spec is the honest degenerate.
        _ => &MARKDOWN,
    }
}

static MARKDOWN: LangSpec = LangSpec {
    fn_kinds: &[],
    fn_required_fields: &[],
    param_list_kinds: &[],
    cc_kinds: &[],
    cc_operators: &[],
    chain_kinds: &[],
    coc_nesting_kinds: &[],
    if_kinds: &[],
    coc_flat_kinds: &[],
    coc_nest_only_kinds: &[],
    coc_operators: &[],
    coc_jump_kinds: &[],
    label_kinds: &[],
    comment_kinds: &[],
    name_style: NameStyle::Any,
    literal_delims: &[],
    call_kinds: &[],
    call_fields: ("function", None),
    call_name_kinds: &[],
    call_member_kinds: &[],
    call_self_words: &[],
    call_member_scopes: &[],
    owner_kinds: &[],
    overloads: None,
    call_import_kinds: &[],
    opaque_fields: &[],
};
