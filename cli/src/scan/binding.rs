//! How Lua and R give a function VALUE its name (plan v2.30 step 4;
//! booklet §4, the unit-name row). Both languages write most named
//! functions as an anonymous `function_definition` bound by the
//! statement around it: a Lua table field (`{ f = function() end }`),
//! a Lua assignment (`local f = function() end`, `M.g, h = function()
//! end, function() end`), an R assignment (`f <- function(x) x`,
//! `(function(x) x) -> f`). One reader of that structure, for the three
//! things it decides:
//!   - the NAME the unit takes (functions::name_of);
//!   - the SCOPE the name lives in — the container a bare call must sit
//!     inside to reach the unit (callees::container_of): the block
//!     around the statement, or the table a field sits in;
//!   - for Lua's `local`, the byte the name is VISIBLE from. A local's
//!     scope begins after its declaration statement, while `local
//!     function f` binds `f` before its body (Lua 5.4 manual §3.5 and
//!     §3.4.11), so `local f = function() f() end` calls whatever `f`
//!     was before — never itself — and a local declared further down
//!     is not yet in scope for the functions above it.
//!
//! No other grammar parents a `function_definition` with these kinds
//! (Python's and the C family's sit in a module, a block, a class body
//! or a template), and no other grammar's `function_declaration` has a
//! `local` token, so every other language reads None by construction.
//! A member-shaped name (`M.f`, `M:f`, `x$f`) also splits into the
//! object it is selected off and the member (`member_of`): the object
//! is the owner the recursion reader's member road keys by.

use super::ast;
use tree_sitter::Node;

/// A name a statement gives a function value (module doc).
pub(crate) struct Binding<'t> {
    /// The node spelling the name: an identifier, a string key or
    /// target, or a member expression.
    pub name: Node<'t>,
    /// The statement making the binding: the declaration, the
    /// assignment (a `local` one's variable_declaration), the field, or
    /// an R chain's outermost assignment — what roxygen documents.
    pub statement: Node<'t>,
    /// The node the name lives in.
    pub scope: Node<'t>,
    /// The byte the name is visible from, where that is not the whole
    /// scope (Lua `local`).
    pub from: Option<usize>,
}

impl Binding<'_> {
    /// A Lua `local`: the one binding whose name is visible from partway
    /// through its scope, and hidden from other files.
    pub(crate) fn is_local(&self) -> bool {
        self.from.is_some()
    }
}

/// The binding that names `node`, when a Lua or R statement binds it.
pub(crate) fn of<'t>(node: Node<'t>, src: &[u8]) -> Option<Binding<'t>> {
    match node.kind() {
        "function_declaration" => local_declaration(node),
        "function_definition" => {
            // parentheses change nothing a binding reads, and R can
            // right-assign a function value only through them
            let mut value = node;
            while let Some(p) = value
                .parent()
                .filter(|p| p.kind() == "parenthesized_expression")
            {
                value = p;
            }
            let parent = value.parent()?;
            match parent.kind() {
                "field" => field(value, parent),
                "expression_list" => assigned(value, parent),
                "binary_operator" => r_assigned(value, parent, src),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A member-shaped unit name split into the object it is selected off
/// and the member (module doc): the owner and the base name the
/// recursion reader's member road keys by. None for a plain name.
pub(crate) fn member_of(node: Node<'_>, src: &[u8]) -> Option<(String, String)> {
    split(name_node(node, src)?, src)
}

/// The node spelling a unit's name: its own named `name` field (R's
/// `function_definition.name` is the keyword token and does not
/// count), else the binding's.
fn name_node<'t>(node: Node<'t>, src: &[u8]) -> Option<Node<'t>> {
    node.child_by_field_name("name")
        .filter(|n| n.is_named())
        .or_else(|| of(node, src).map(|b| b.name))
}

/// The text a name node spells: a string key or target by its content
/// (`["f"] = …` and `"f" <- …` both name `f`), anything else as
/// written. An empty string names nothing.
pub(crate) fn spelled(name: Node<'_>, src: &[u8]) -> Option<String> {
    let node = if name.kind() == "string" {
        name.child_by_field_name("content")?
    } else {
        name
    };
    node.utf8_text(src).ok().map(str::to_string)
}

/// `M.f` and `M:f` (Lua), `x$f` and `x@f` (R) as (object, member). A
/// Lua `M["f"] = …` keeps its bracket spelling whole: the rare form is
/// a name no member road keys by — the undercounting direction.
fn split(name: Node<'_>, src: &[u8]) -> Option<(String, String)> {
    let (object, member) = match name.kind() {
        "dot_index_expression" => ("table", "field"),
        "method_index_expression" => ("table", "method"),
        "extract_operator" => ("lhs", "rhs"),
        _ => return None,
    };
    let text = |field| spelled(name.child_by_field_name(field)?, src);
    Some((text(object)?, text(member)?))
}

/// `local function f`: the name is the declaration's own, visible from
/// the declaration on, its body included (manual §3.4.11).
fn local_declaration(node: Node<'_>) -> Option<Binding<'_>> {
    if !ast::children(node).iter().any(|c| c.kind() == "local") {
        return None;
    }
    Some(Binding {
        name: node.child_by_field_name("name")?,
        statement: node,
        scope: node.parent()?,
        from: Some(node.start_byte()),
    })
}

/// `{ f = function() end }` and `{ ["f"] = function() end }`: the key
/// names the value; a computed key names nothing — the grammar gives
/// `[k] = …` the same `name: identifier` as `k = …`, so inside brackets
/// only a string counts. The table is the scope — a member scope
/// (LangSpec::call_member_scopes).
fn field<'t>(node: Node<'t>, field: Node<'t>) -> Option<Binding<'t>> {
    if field.child_by_field_name("value")?.id() != node.id() {
        return None;
    }
    let bracketed = ast::children(field).iter().any(|c| c.kind() == "[");
    let key = if bracketed { "string" } else { "identifier" };
    let name = field
        .child_by_field_name("name")
        .filter(|n| n.kind() == key)?;
    Some(Binding {
        name,
        statement: field,
        scope: field.parent()?,
        from: None,
    })
}

/// `f, M.g = function() end, function() end`: the value at position i
/// takes the variable at position i. `local` wraps the statement in a
/// variable_declaration, after which the name is visible.
fn assigned<'t>(node: Node<'t>, list: Node<'t>) -> Option<Binding<'t>> {
    let statement = list
        .parent()
        .filter(|s| s.kind() == "assignment_statement")?;
    let at = fielded(list, "value")
        .iter()
        .position(|v| v.id() == node.id())?;
    let variables = ast::children(statement)
        .into_iter()
        .find(|c| c.kind() == "variable_list")?;
    let name = *fielded(variables, "name").get(at)?;
    let parent = statement.parent()?;
    Some(if parent.kind() == "variable_declaration" {
        Binding {
            name,
            statement: parent,
            scope: parent.parent()?,
            from: Some(parent.end_byte()),
        }
    } else {
        Binding {
            name,
            statement,
            scope: parent,
            from: None,
        }
    })
}

/// `f <- function(x) x` (also `=`, `<<-`, `:=`) names the value by its
/// left side, `(function(x) x) -> f` (also `->>`) by its right —
/// without the parentheses R, and the grammar, read `function(x) x ->
/// f` as a body that assigns `x` to `f`. A chained
/// `f <- g <- function()` names it `g` and lives where the outermost
/// statement does. `<<-` assigns in an enclosing environment, which a
/// bare call reaches only after the assigning function ran; the
/// statement's own block is the scope, the narrower — undercount —
/// reading.
fn r_assigned<'t>(node: Node<'t>, op: Node<'t>, src: &[u8]) -> Option<Binding<'t>> {
    let name = target(op, node, src)?;
    let mut statement = op;
    while let Some(outer) = statement
        .parent()
        .filter(|p| target(*p, statement, src).is_some())
    {
        statement = outer;
    }
    Some(Binding {
        name,
        statement,
        scope: statement.parent()?,
        from: None,
    })
}

/// The name an assignment `op` gives its operand `value`, when `op` is
/// an assignment of `value` to a name: an identifier, a string or a
/// member (`x$f`). Any other target (`x[["f"]]`, `attr(x, "f")`) names
/// nothing.
fn target<'t>(op: Node<'t>, value: Node<'t>, src: &[u8]) -> Option<Node<'t>> {
    if op.kind() != "binary_operator" {
        return None;
    }
    let (from, to) = match ast::operator_text(op, src)? {
        "<-" | "=" | "<<-" | ":=" => ("rhs", "lhs"),
        "->" | "->>" => ("lhs", "rhs"),
        _ => return None,
    };
    if op.child_by_field_name(from)?.id() != value.id() {
        return None;
    }
    op.child_by_field_name(to)
        .filter(|n| matches!(n.kind(), "identifier" | "string" | "extract_operator"))
}

fn fielded<'t>(node: Node<'t>, field: &str) -> Vec<Node<'t>> {
    let mut cursor = node.walk();
    node.children_by_field_name(field, &mut cursor).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/scan/binding.rs"]
mod tests;
