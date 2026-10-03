-- | The `ce churn` console lines (plan v2.32 step 5), transcribed from
-- the Rust printer this replaced (cli/src/churn/report.rs
-- `print_console`, deleted at step 5 R0): the window's commits and
-- ledger sums, survival, the strongest co-change pairs up to the
-- display cut with the rest counted, the commits too large to pair
-- and the submodules without file history. Report-only: no veto.
module CE.Churn.Lines (lines') where

import CE.Churn.Document (cochangeFileCap, displayCut)
import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) $
    [ say "window" [N (fact req "days"), n "commits", n "append_lines", n "rewrite_lines"]
    , say "survival" [n "surviving", n "added_in_window", n "churned"]
    ]
      <> [say "pair" [N (int "count" p), R (key "a" p), R (key "b" p)] | p <- take displayCut pairs]
      <> [say "more" [N (toInteger (length pairs - displayCut))] | length pairs > displayCut]
      <> [say "skipped" [n "skipped_large_commits", N cochangeFileCap] | int "skipped_large_commits" doc > 0]
      <> [say "submodules" [P (join ", " (map (piece . R) subs))] | not (null subs)]
 where
  n k = N (int k doc)
  pairs = arr "cochange" doc
  subs = arr "submodules_without_file_history" doc
