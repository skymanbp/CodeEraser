-- | The html definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec.rs at a378e78c; from this commit on
-- the core is the authority.
module CE.Lang.Html where

top :: String
top =
  "name = 'html'\n"

scan :: String
scan =
  "# HTML (plan v2.30 step 5): a document language like Markdown — no\n\
  \# functions, no metrics — whose one grammar fact a spec reader wants\n\
  \# is its comment node kind (booklet §4; the second-interpreter\n\
  \# reader's comment test). Everything else is Markdown's empty table.\n\
  \comment_kinds = ['comment']\n"
