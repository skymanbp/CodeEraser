-- | The cpp definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec_c.rs, cli/src/flow/spec_c.rs,
-- cli/src/merge/slot_c.rs at e877f389; from this commit on the core is the
-- authority.
module CE.Lang.Cpp where

name :: String
name =
  "name = 'cpp'\n"

overloads :: String
overloads =
  "# C++: a name is overloaded by its parameters. A default argument\n\
  \# counts toward the upper bound only; a parameter pack and the\n\
  \# C-style `...` (an anonymous token in a parameter_list, probed)\n\
  \# remove it; a pack expansion in a call (`f(a...)`, probed as\n\
  \# parameter_pack_expansion) passes a count no reader can tell. A\n\
  \# constructor stays reachable — `K(x)` constructs a K.\n\
  \overloads = {optional = ['optional_parameter_declaration'], variadic = ['variadic_parameter_declaration', '...'], ignored = [], spread = ['parameter_pack_expansion'], unreachable = []}\n"

std :: String
std =
  "# C++ adds its three `std::` spellings to C's names.\n\
  \  'std::exit', 'std::abort', 'std::terminate',\n"

only :: String
only =
  "[[slot]]\n\
  \expr_kinds = ['lambda_expression', 'this', 'qualified_identifier', 'compound_literal_expression']\n\
  \type_kinds = ['auto', 'placeholder_type_specifier', 'template_type', 'template_argument_list']\n\
  \name_fields = [['optional_parameter_declaration', 'declarator']]\n\
  \other_kinds = [\n\
  \  'abstract_function_declarator', 'namespace_identifier', 'condition_clause',\n\
  \  'lambda_capture_specifier', 'lambda_default_capture', 'optional_parameter_declaration',\n\
  \  'reference_declarator', 'structured_binding_declarator', 'subscript_argument_list',\n\
  \  'field_initializer_list', 'field_initializer',\n\
  \]\n"
