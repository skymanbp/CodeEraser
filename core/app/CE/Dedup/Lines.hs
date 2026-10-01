-- | The `ce dedup` console lines (plan v2.32 step 5), transcribed from
-- cli/src/dedup/report.rs `print_console` and cli/src/dedup/budget.rs
-- `check`: one line per clone block, the index summary, and under
-- `--check` (the `check` fact) the ratchet's line — over the budget on
-- stderr, under it the advice to ratchet down on stdout, at it none.
-- The veto: `--check` and verdict/1's fail bit (the budget held).
module CE.Dedup.Lines (lines', veto) where

import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) (map dup (arr "blocks" doc) <> [say "summary" (map (N . (`int` summary)) order)])
    <> [l | flag req "check", l <- ratchet]
 where
  summary = key "summary" doc
  order = words "files refreshed removed blocks groups min_tokens min_distinct low_diversity_suppressed hot_chained stale_skipped"
  dup b = say "dup" ([R (key "a_file" b)] <> span' "a" b <> [R (key "b_file" b)] <> span' "b" b <> [N (int "tokens" b)])
  span' side b = [N (int (side <> "_start") b), N (int (side <> "_end") b)]
  blocks = int "blocks" summary
  budget = fact req "budget"
  ratchet
    | flag req "fail" = [line 1 (say "over" [N blocks, N budget])]
    | blocks < budget = [line 0 (say "under" [N blocks, N budget])]
    | otherwise = []

-- | `--check` with the budget's fail bit.
veto :: Value -> DocReq -> Bool
veto _ req = flag req "check" && flag req "fail"
