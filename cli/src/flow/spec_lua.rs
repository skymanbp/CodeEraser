//! The Lua flow table (plan v2.31 step 4), beside the contract in
//! spec.rs; kinds read off the round-3 probe transcript of flow.lua.
//! Lua spells every name `identifier` — a field, a method, a label, a
//! `<const>` attribute — so the name positions carry most of the table;
//! an if branch may hold no block at all (§5.3: the empty flag), and a
//! `repeat … until` condition exits the loop (§5.1 rule 5).

use super::spec::Table;

pub static LUA: Table = Table::new(
    "lua",
    &[r#"
block_kinds = ["block"]
wrapper_kinds = ["do_statement"]
if = {cond = "condition", then = "consequence", else = "alternative"}
elif_kinds = ["elseif_statement"]
else_kinds = [["else_statement", "body"]]
loops = [
  {kind = "for_statement", header = "@for_numeric_clause", target = "name", iter = "*", body = "body"},
  {kind = "for_statement", header = "@for_generic_clause", target = "@variable_list", iter = "@expression_list", body = "body"},
  {kind = "while_statement", cond = "condition", body = "body"},
  {kind = "repeat_statement", cond = "condition", body = "body", body_first = true, until = true},
]
return_kinds = ["return_statement"]
break_kinds = ["break_statement"]
gotos = [["goto_statement", "@identifier"]]
labels = [["label_statement", "@identifier", ""]]
noreturn = ["error", "os.exit"]
const_true = [["true", ""]]
const_false = [["false", ""], ["nil", ""]]
int_kinds = ["number"]
dynamic_names = ["load", "loadstring", "dofile", "setfenv", "getfenv", "debug.*"]
decls = [{kind = "variable_declaration", binder = "@variable_list|@assignment_statement/@variable_list", init = "@assignment_statement/@expression_list"}]
pattern_kinds = ["variable_list"]
ident_kinds = ["identifier"]
name_positions = [
  ["dot_index_expression", "field"], ["method_index_expression", "method"], ["field", "name"],
  ["variable_list", "attribute"], ["goto_statement", "@identifier"], ["label_statement", "@identifier"],
]
member_write_bases = ["dot_index_expression", "bracket_index_expression"]
assigns = [{kind = "assignment_statement", left = "@variable_list", right = "@expression_list"}]
conditional_ctx = [["binary_expression", "right", ["and", "or"]]]
"#],
);

#[cfg(test)]
#[path = "../../tests/unit/flow/lang_lua.rs"]
mod lang_lua;
