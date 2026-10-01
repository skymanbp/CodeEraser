-- | The tsx definition tables (plan v2.32 step 1),
-- transcribed from cli/src/merge/slot_ts.rs at e877f389; from this commit
-- on the core is the authority.
module CE.Lang.Tsx where

name :: String
name =
  "name = 'tsx'\n"

only :: String
only =
  "[[slot]]\n\
  \expr_kinds = ['jsx_element', 'jsx_self_closing_element', 'jsx_text']\n\
  \other_kinds = ['jsx_opening_element', 'jsx_closing_element', 'jsx_attribute', 'jsx_expression']\n"
