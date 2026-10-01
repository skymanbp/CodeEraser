-- | The `ce clone` console lines (plan v2.32 step 5): the T3 report
-- through the shared throat (cli/src/report.rs `emit` with the
-- templates of cli/src/dedup/t3/mod.rs `print`) — one line per clone
-- pair, the summary with the counters it does not name as a raw tail —
-- and the `--units` listing (cli/src/main_judge.rs `print_units`). Both
-- report-only: no veto.
module CE.Clone.Lines (lines', unitLines) where

import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ = emitted say hit "clones" (words "clones units judged cached prefiltered") doc
 where
  hit c = [R (key "a" c), R (key "b" c), N (int "ted" c), N (int "n1" c), N (int "n2" c)]

unitLines :: Say -> Value -> DocReq -> [Line]
unitLines say doc _ =
  map (line 0) $
    [say "unit" [R (key "path" u), R (key "key" u), N (int "nth" u), N (int "nodes" u)] | u <- units]
      <> [say "units" [N (toInteger (length units))]]
 where
  units = arr "units" doc
