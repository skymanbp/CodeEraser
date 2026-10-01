-- | The markdown definition tables (plan v2.32 step 1),
-- transcribed from cli/src/scan/spec.rs at a378e78c; from this commit on
-- the core is the authority.
module CE.Lang.Markdown where

top :: String
top =
  "name = 'markdown'\n"

scan :: String
scan =
  "# Markdown (and HTML after it) states no unit, metric or call kind:\n\
  \# every field but the call fields is the empty table.\n\
  \[scan]\n\
  \call_fields = ['function', null]\n"
