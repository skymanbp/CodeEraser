//! The rows a flow table is made of (plan v2.31 step 4; design
//! booklet docs/reference/analysis-track.md §5.1): their shapes and
//! the choices they make. Beside spec.rs for the reason scan::spec_c
//! gives for its own split: the contract read past the 300-line line
//! once every row carried its documentation. A field written without
//! a type is a position (pos.rs).

use super::pos::Pos;
use serde::{Deserialize, Serialize};

/// Node kinds — or, where a field says so, anonymous tokens.
pub type Kinds = Vec<String>;
/// Names as the source spells them: a callee, a parameter.
pub type Names = Vec<String>;
/// A kind and one string about it: a position, a token or a name.
pub type Pairs = Vec<(String, String)>;
/// A kind and two positions.
pub type Triples = Vec<(String, Pos, Pos)>;
/// Declaring tokens, each with the scope it gives (Loop::marker).
pub type Marks = Vec<(String, Scope)>;
/// A kind, a position in it, and the operator tokens that make the
/// position conditional (none listed: every node of the kind does).
pub type Ops = Vec<(String, Pos, Kinds)>;

/// Every table struct in one expansion: one derive and one serde
/// reading — an absent key is a part the construct lacks (`default`), a
/// misspelt key refuses the table at load (`deny_unknown_fields`) — and
/// a field written without a type is a position. One macro rather than
/// a header per struct: the clone gate reads a derive over a run of
/// `pub name: Type,` fields as a copy of every other struct shaped so
/// (config::rules::ClassKnobs was the first it named).
macro_rules! rows {
    (@ty) => { $crate::flow::pos::Pos };
    (@ty $ty:ty) => { $ty };
    ($($(#[$doc:meta])* $name:ident {
        $($(#[$fdoc:meta])* $field:ident $(: $ty:ty)?),* $(,)?
    })*) => {$(
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $name {
            $($(#[$fdoc])* pub $field: $crate::flow::rows::rows!(@ty $($ty)?),)*
        }
    )*};
}
pub(crate) use rows;

/// The choices a row makes, each read by its lowercase name; the
/// `#[default]` variant is the choice of a row that names none.
macro_rules! choice {
    ($($(#[$doc:meta])* $name:ident { $($(#[$vdoc:meta])* $variant:ident),* $(,)? })*) => {$(
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
        #[serde(rename_all = "lowercase")]
        pub enum $name { $($(#[$vdoc])* $variant,)* }
    )*};
}

choice! {
    /// Where a declaration is visible (§5.1 rule 8).
    Scope {
        /// The enclosing block: shadowing, and gone at its end.
        #[default]
        Block,
        /// The whole unit (Python and R, TypeScript `var`).
        Function,
    }
    /// What an assignment does to its target (rules 8 and 9).
    Mode {
        /// The value is read, then the target written (mode 1).
        #[default]
        Write,
        /// The target is read and written (mode 2): `x += v`.
        ReadWrite,
        /// The value is read; the target is no local of this unit,
        /// neither written nor declared (R `<<-`, rule 8).
        Outer,
    }
    /// Whether an arm runs into the next one: its case's fallthrough
    /// bit (rule 6).
    Fall {
        /// Rust and Python arms, Java arrow rules.
        #[default]
        Never,
        /// C, C++, TypeScript and Java colon cases: every arm, the
        /// break a statement of its own.
        Always,
        /// When the arm's last statement is of a
        /// FlowSpec::fallthrough_kinds kind (Go).
        Statement,
    }
}

/// When an arm is its switch's default, giving the switch its has_else
/// bit (rule 6).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DefaultArm {
    /// Never (a TypeScript `case`; a Rust arm — Switch::always_default
    /// speaks for the whole match).
    #[default]
    Never,
    /// By its kind (TypeScript switch_default, Go default_case).
    Kind,
    /// When nothing sits at this position: a C case_statement without
    /// a `value` is `default:`.
    Missing(Pos),
    /// When a node at this position holds this token as a direct child
    /// (Python `case _:`, Java `default`). A guarded wildcard is no
    /// default: the lowering reads the guard first.
    Holds(Pos, String),
}

rows! {
    /// The positions of an if, read on each LangSpec::if_kinds node —
    /// the kinds are scan's, verified there (rule 2).
    If {
        /// The condition, read in the head.
        cond,
        /// The branch taken when it holds.
        then,
        /// The else part: a FlowSpec::else_kinds wrapper, an elif, the
        /// next if, or the else statement itself (Go, Java, R).
        r#else,
        /// A statement run before the condition, its names scoped to
        /// the if (Go `if v := f(); v > 0`).
        init,
    }
    /// A loop form (rules 2, 4, 5 and 8). Rows sharing a kind are tried
    /// in order: the first whose `header` gives a node applies, and a
    /// row without a header takes the rest.
    Loop {
        kind: String,
        /// A child holding the head (Go's for_clause, Lua's clauses):
        /// `cond`, `init`, `update`, `target` and `iter` are read on it.
        header,
        body,
        /// Checked before each round. None with no iterable, an empty
        /// one, or a FlowSpec::const_true literal = infinite (rule 5).
        cond,
        /// A C-style init: a sibling before the loop (rule 4).
        init,
        /// A C-style update: a label ending the body, the loop's own
        /// continues turned into gotos to it (rule 4).
        update,
        /// A for-in or range target, written each round (rule 8).
        target,
        /// A for-in or range iterable, read once in the head.
        iter,
        /// Tokens that make `target` a declaration, each with its scope,
        /// when the node holding the target has one as a direct child
        /// (TypeScript `const` / `var`, Go `:=`); without one it writes
        /// a variable declared before. Empty: the target declares in the
        /// loop — or, where a first write declares, is written.
        marker: Marks,
        /// The body runs before the first check (do-while, repeat).
        body_first: bool,
        /// The condition exits the loop (Lua `until`): infinite on a
        /// FlowSpec::const_false literal (rule 5).
        until: bool,
        /// Python's `else:` block: loop · block · label, the loop's own
        /// breaks turned into gotos to the label (rule 4).
        r#else,
    }
    /// A switch or match form (rules 2 and 6).
    Switch {
        kind: String,
        /// Read in the head ("" = none: Go's select).
        subject,
        /// The node whose children with a FlowSpec::cases row are the
        /// arms; `.` = the switch holds them itself (Go).
        arms,
        /// A statement run before the subject, scoped to the switch.
        init,
        /// Names the head binds in every arm (Go `v := x.(type)`).
        binder,
        /// has_else whatever the arms say (Rust match, Go select).
        always_default: bool,
        /// Without arms the switch never returns: kind 15 (Go
        /// `select {}`).
        empty_noreturn: bool,
        /// A break inside an arm leaves the enclosing loop: the match is
        /// no break target (Rust, Python).
        passes_break: bool,
    }
    /// An arm form (rules 2, 6, 8 and 9).
    Case {
        kind: String,
        default: DefaultArm,
        fallthrough: Fall,
        /// The pattern: its names declared in the arm and written in
        /// its head.
        pattern,
        /// Values the arm tests, read in the head.
        value,
        /// The guard, read in the head once the pattern has bound.
        guard,
        body,
    }
    /// A try form (rule 2). Its catches and finally are its children
    /// with a FlowSpec::catches or FlowSpec::finally_kinds row.
    Try {
        kind: String,
        body,
        /// Python's `else:`: the body's second block child.
        r#else,
        /// Java's resources, declared and read in the head (rule 9).
        resources,
    }
    /// A catch form (rules 2, 8 and 9).
    Catch {
        kind: String,
        /// The binding written on entry, declared in the catch.
        param,
        /// Read on entry: a Python except's exception types.
        value,
        body,
    }
    /// A block with a head (Python `with`): each item's `value` read and
    /// `binder` written in turn (rule 8's `with … as`), then the body.
    With {
        kind: String,
        item,
        value,
        binder,
        body,
    }
    /// A declaration form (rule 8).
    Decl {
        kind: String,
        /// A token the node must hold as a direct child ("" = none
        /// asked): Go's `:=` on a receive statement.
        token: String,
        scope: Scope,
        /// The per-binding items ("" = the node is its one item).
        items,
        /// The item kind pairing a binder with an initializer; an item
        /// of another kind is its own binder ("" = every item pairs).
        pair: String,
        /// On a pair: the name, pattern or declarator chain bound.
        binder,
        /// On a pair: the initializer, read before the binder is written.
        init,
        /// A storage-class child: a FlowSpec::lasting_storage token
        /// there makes the names no locals (C `static`).
        storage,
        /// The block a refuted pattern runs, which must leave (Rust
        /// let-else).
        alternative,
        /// A name this scope already declares is written, not declared
        /// again (Go `:=`).
        redeclare: bool,
    }
    /// An assignment form (rules 8 and 9); rows are tried in order.
    Assign {
        kind: String,
        /// The operator token the node must hold ("" = any).
        op: String,
        left,
        /// Read first (rule 9: right before left).
        right,
        mode: Mode,
    }
    /// A macro form (rules 3 and 9): the name with its `!` is the callee
    /// the name tables match; every identifier in the arguments is read,
    /// and so is every name a placeholder of a string of a `strings`
    /// kind there reads (format.rs).
    Macro {
        kind: String,
        name,
        args,
        strings: Kinds,
    }
}
