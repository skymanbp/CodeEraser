//! The syntax of a POSITION (plan v2.31 step 4; design booklet
//! docs/reference/analysis-track.md §5.1): where, in a node of a row's
//! kind, one part of a construct sits. A position gives NAMED nodes
//! only (the `;` a TypeScript `for` keeps on its `condition` field is
//! no part):
//! - `""` — the construct lacks the part;
//! - `name` — the children on that field (one field may hold several:
//!   Python hangs every elif and the else on `alternative`);
//! - `@kind` — the children of that kind, on a field or not: the
//!   grammars leave many parts unnamed (a Python except's block, a Go
//!   case's statement list), and LangSpec::coc_nesting_kinds names such
//!   a body by its kind the same way;
//! - `a/b` — a path: `b` read on each node `a` gives (Rust hangs an
//!   arm's guard inside its pattern: `pattern/condition`);
//! - `a|b` — the first alternative that gives a node (Lua holds the
//!   names of `local x` and of `local x = v` at two depths);
//! - `*` — the children no other position of the row names at the same
//!   node, comments excluded (the one statement a TypeScript `else`
//!   holds, a C case's statements after its value);
//! - `.` — the node itself.
//!
//! Where one position of a row reaches inside another's node, the inner
//! node belongs to the inner position alone: a Python except's `value`
//! holds its binding at `value/alias`, and the binding is written, never
//! read with the exception type. Resolving positions against a tree is
//! the lowering's work (step 4 A2); this module owns their syntax, and
//! the unit legs read every field and kind a position names against
//! its grammar.

/// A position, in the syntax above.
pub type Pos = String;

/// One step of a position path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step<'a> {
    /// The children on this field.
    Field(&'a str),
    /// The children of this kind.
    Kind(&'a str),
}

/// A position's alternatives, each a path of steps; `""`, `*` and `.`
/// name no field and no kind.
pub fn paths(position: &str) -> Vec<Vec<Step<'_>>> {
    if matches!(position, "" | "*" | ".") {
        return Vec::new();
    }
    position
        .split('|')
        .map(|alt| alt.split('/').map(step).collect())
        .collect()
}

fn step(text: &str) -> Step<'_> {
    text.strip_prefix('@').map_or(Step::Field(text), Step::Kind)
}
