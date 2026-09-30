//! The flow family's per-language tables (plan v2.31 step 4; design
//! booklet docs/reference/analysis-track.md §5.1 and §5.3). This file is
//! the CONTRACT — the struct, the table type and the dispatch — with the
//! tables beside it, one file per language family as scan::spec keeps
//! its own. A table is TOML text read once into a FlowSpec, a misspelt
//! key refused by name: to the clone gate one table is one string, so
//! ten tables of one shape cannot read as copies of one another.
//!
//! What scan's LangSpec already answers is read from
//! scan::spec::spec(lang), never copied: fn_kinds, param_list_kinds,
//! if_kinds, coc_jump_kinds, label_kinds (a jump's label child),
//! call_kinds, call_fields, opaque_fields and comment_kinds. Each field
//! below names the §5.1 rule it encodes: the table holds the grammar's
//! facts, the lowering (step 4 A2) applies the rule.

use super::rows::{
    Assign, Case, Catch, Decl, If, Kinds, Loop, Macro, Names, Ops, Pairs, Scope, Switch, Triples,
    Try, With, rows,
};
use super::{spec_c, spec_go, spec_java, spec_lua, spec_py, spec_r, spec_rs, spec_ts};
use crate::scan::lang::Lang;
use std::sync::OnceLock;

rows! {
    /// One language's flow table (§5.3's rows, as its grammar spells them).
    FlowSpec {
        /// Rule 1: statement containers. A2 lowers kind 0 (empty flag when
        /// none is left) and each named child, comments aside, as a statement.
        block_kinds: Kinds,
        /// Rule 1: children of a container whose own children are its
        /// statements (Go's statement_list): A2 splices them in.
        splice_kinds: Kinds,
        /// Rule 1: statements that are their one named child (an expression
        /// statement, Rust `unsafe { }`, Lua `do … end`): A2 lowers the child.
        wrapper_kinds: Kinds,
        /// Rule 5: kinds that spell nothing: a loop part of one is absent
        /// (TypeScript `for (;;)` holds empty statements there).
        empty_kinds: Kinds,
        /// Rule 2: where an if keeps its parts (the kinds are LangSpec's).
        /// A2 gives it then [+ else], one child each.
        r#if: If,
        /// Rule 2: continuations with a condition, parts at the If positions
        /// (Python elif, Lua elseif): A2 nests them as ifs in else position.
        elif_kinds: Kinds,
        /// Rule 2: (kind, body) of a node wrapping a final else: A2 unwraps it.
        else_kinds: Pairs,
        /// Rule 5: (kind, value, init) of a node around a condition: A2 reads
        /// a constant through it and runs the init first (C++ `if (T v = …; c)`).
        cond_wrappers: Triples,
        /// Rules 2, 4, 5 and 8: loop forms, kind 3.
        loops: Vec<Loop>,
        /// Rules 2 and 6: switch and match forms, kind 4, one case per arm.
        switches: Vec<Switch>,
        /// Rules 2, 6, 8 and 9: arm forms, kind 5.
        cases: Vec<Case>,
        /// Rule 2: try forms, kind 6 — body · catch* · finally?.
        tries: Vec<Try>,
        /// Rules 2 and 8: catch forms, kind 7.
        catches: Vec<Catch>,
        /// Rule 2: (kind, body) of a finally, kind 8.
        finally_kinds: Pairs,
        /// Rule 8's `with … as`: a block with a head. Rule 2 lists no kind
        /// for it; without the row A2 finds no body.
        withs: Vec<With>,
        /// Rule 3: kind 9.
        return_kinds: Kinds,
        /// Rule 3: kind 10 (throw, raise).
        throw_kinds: Kinds,
        /// Rule 3: kind 11, aux the enclosing loop or switch — or, labelled
        /// at a block, a goto to a label after it (rule 4).
        break_kinds: Kinds,
        /// Rule 3: kind 12 (continue, R `next`), aux the enclosing loop.
        continue_kinds: Kinds,
        /// Rule 3: (kind, label) of a goto, kind 13, aux the label's seq.
        gotos: Pairs,
        /// Rule 3: (kind, name, statement) of a label, kind 14 ("" statement:
        /// a bare label, Lua `::name::`).
        labels: Triples,
        /// Rule 4: where a loop or block holds its own label (Rust
        /// `'a: loop`): a labelled block lowers to block · label.
        self_label,
        /// Rule 4: a break to the enclosing switch, its value read (Java yield).
        yield_kinds: Kinds,
        /// Rules 4 and 6: an arm ending in one sets its case's fallthrough
        /// flag (Go).
        fallthrough_kinds: Kinds,
        /// Rule 3: callee names, verbatim: an expression statement whose
        /// top-level call names one is kind 15.
        noreturn: Names,
        /// Rule 3: (kind, name): a declaration in the file holding a node of
        /// this kind whose text names this is noreturn (C `_Noreturn`).
        noreturn_attrs: Pairs,
        /// Rule 3: callee names that return (R `return(x)`): kind 9.
        return_calls: Names,
        /// Rules 3 and 7: (kind, callee) of calls beside LangSpec::call_kinds
        /// (TypeScript `new Function(…)`).
        call_forms: Pairs,
        /// Rules 3 and 9: macro forms (Rust).
        macros: Vec<Macro>,
        /// Rule 5: (kind, token): a literal true — the kind itself, or the
        /// kind holding the token. A loop checking one is infinite.
        const_true: Pairs,
        /// Rule 5: likewise false: Lua `repeat … until false` is infinite.
        const_false: Pairs,
        /// Rule 5: integer literals: A2 reads one whose digits are not all
        /// zero as true.
        int_kinds: Kinds,
        /// Rule 7: callee names (an entry ending `.*` covers every name
        /// below it: Lua `debug.*`); a call to one sets the unit's dynamic flag.
        dynamic_names: Names,
        /// Rule 7: kinds whose presence in the body sets it (C `#if`, asm).
        dynamic_kinds: Kinds,
        /// Rule 8: the scope a block opens: Block (shadowing) or Function
        /// (none: Python, R).
        scoping: Scope,
        /// Rule 8: a name first bound by a write is a local (Python, R).
        first_write_declares: bool,
        /// Rule 8: (kind, binder) in the parameter list: an ident_kinds child
        /// binds itself, a pattern by its names, any other kind (Rust self,
        /// Java's receiver, TypeScript `this`) nothing.
        params: Pairs,
        /// Rule 8: a first parameter so named is ignored (Python self, cls).
        receiver_names: Names,
        /// Rule 8 (ruling 6): where named results sit (Go): parameters
        /// too, each read by a bare return.
        results,
        /// Rule 8: declaration forms.
        decls: Vec<Decl>,
        /// Rule 8: storage tokens that make a declaration's names no locals.
        lasting_storage: Kinds,
        /// Rule 8: destructuring kinds: reached from a binder, an ident_kinds
        /// node in one binds, and a node of any other kind is read.
        pattern_kinds: Kinds,
        /// Rule 8: other kinds that bind their text in a pattern (`{ x }`).
        pattern_idents: Kinds,
        /// Rule 8: in a pattern one of these binds its name when it has one,
        /// and reads a value when dotted (Python `case a.b`).
        dotted_patterns: Kinds,
        /// Rule 8: a capitalised name in a pattern is a path (Rust `None`).
        upper_pattern_paths: bool,
        /// Rule 8: (kind, inner) of a declarator a binder is read through.
        binder_paths: Pairs,
        /// Rule 8 (ruling 7): a declarator of this kind at a binder's top
        /// whose own declarator is a bare name declares a function (a
        /// prototype in a body), no local.
        prototype_kinds: Kinds,
        /// Rule 8: (kind, pattern) of an expression that binds (Rust
        /// `if let`, Java `instanceof T t`), declared at its statement.
        pattern_binders: Pairs,
        /// Rule 8: the pattern_binders kinds whose names live only in the
        /// branch or loop body they guard (Rust `if let` — an else may
        /// read an outer name so spelled); the others bind in the
        /// statement's own scope, where Java's flow scoping reaches.
        branch_binders: Kinds,
        /// Rule 8: statements whose names are no locals (Python global).
        nonlocal_kinds: Kinds,
        /// Rule 8: scopes whose bindings are not the unit's (Python
        /// comprehensions and lambda); host names inside are still read.
        local_only_scopes: Kinds,
        /// Rule 9: the kinds of a name. Outside name_positions one resolves
        /// to the nearest visible declaration, or to nothing.
        ident_kinds: Kinds,
        /// Rule 9: (kind, part): where a name is no variable (a member, key,
        /// keyword argument, type or label).
        name_positions: Pairs,
        /// Rule 9: other kinds read as the variable of their text (`{ x }`).
        shorthand_kinds: Kinds,
        /// Rule 9: string kinds whose `{name}` pieces are read — R's glue
        /// and cli strings interpolate the caller's locals by name.
        interpolated_strings: Kinds,
        /// Rule 9: an assignment target of one of these writes nothing, its
        /// names are read (`a.b = v`, `*p = v`).
        member_write_bases: Kinds,
        /// Rules 8 and 9: assignment forms.
        assigns: Vec<Assign>,
        /// Rule 9: (kind, target) of `x++`: mode 2.
        update_kinds: Pairs,
        /// Rule 9: (kind, part, operators): a write there is mode 2 — a
        /// right operand, a branch, an if or match in expression position.
        conditional_ctx: Ops,
        /// Rule 9: (kind, default): read in the unit's first, synthetic
        /// statement.
        default_arg_fields: Pairs,
        /// Rule 10: nested scopes that are no units: a host local they name
        /// is captured.
        capture_kinds: Kinds,
        /// Rule 10: (kind, operator token or `@kind`) taking an address.
        address_ops: Pairs,
        /// Rule 10: declarators binding a reference: the names bound are
        /// address-taken (C++ `T& r = x`).
        ref_binding_kinds: Kinds,
        /// Rule 10: names that are never variables: a write to one is
        /// dropped (Go `_`).
        discard_names: Names,
    }
}

/// A table: TOML text in pieces (C and C++ share every piece but the
/// noreturn names), read into its FlowSpec on first use.
pub struct Table {
    name: &'static str,
    pieces: &'static [&'static str],
    read: OnceLock<FlowSpec>,
}

impl Table {
    pub const fn new(name: &'static str, pieces: &'static [&'static str]) -> Self {
        Self {
            name,
            pieces,
            read: OnceLock::new(),
        }
    }

    /// The table, read once. The text is this crate's own, so a table
    /// that does not read is a build defect the unit legs name — never
    /// an input the caller could have avoided.
    pub fn get(&self) -> &FlowSpec {
        self.read.get_or_init(|| {
            toml::from_str(&self.pieces.concat())
                .unwrap_or_else(|e| panic!("flow table `{}` does not read: {e}", self.name))
        })
    }
}

/// The table of a language whose units the family lowers (§5.3): the
/// judged languages less Markdown, Haskell and HTML. None for those
/// three, the scan-only arm, Text and the sentinel.
pub fn spec(lang: Lang) -> Option<&'static FlowSpec> {
    let table = match lang {
        Lang::Python => &spec_py::PYTHON,
        Lang::TypeScript | Lang::Tsx => &spec_ts::TYPESCRIPT,
        Lang::Rust => &spec_rs::RUST,
        Lang::Go => &spec_go::GO,
        Lang::C => &spec_c::C,
        Lang::Cpp => &spec_c::CPP,
        Lang::Java => &spec_java::JAVA,
        Lang::Lua => &spec_lua::LUA,
        Lang::R => &spec_r::R,
        _ => return None,
    };
    Some(table.get())
}

#[cfg(test)]
#[path = "../../tests/unit/flow/spec.rs"]
mod tests;
