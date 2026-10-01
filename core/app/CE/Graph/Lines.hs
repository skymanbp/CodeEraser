-- | The `ce deadcode` console lines (plan v2.32 step 5), transcribed
-- from cli/src/graph/deadcode/report.rs and the `--check` lines of
-- cli/src/main_cmds.rs: each dead file with its verdict, reason and
-- trust; the aggregates; the symbol advisory when the road was asked
-- (the dropped table in one line, else its rows, the census by code
-- and the producer's cut); the summary, the degraded reason, the
-- `entry_globs` hint when half the file tier or more is dead (two at
-- least — `files` the file nodes); and under `--check` (the `check`
-- fact) the refusal on stderr. The veto: `--check` and the judgment's
-- fail — a dead file, or a degraded run.
module CE.Graph.Lines (lines', veto) where

import CE.Document.Read
import CE.Graph.Cost (unmentionedCap)
import qualified CE.Graph.Document as Dead
import qualified CE.Text.Deadcode as T
import Data.List (nub)

lines' :: Lang -> DocReq -> [Line]
lines' lang req =
  map (line 0) (map dead (arr "dead" doc) <> map aggregate (arr "reported" doc) <> advisory <> tail')
    <> [line 1 l | flag req "check", failed doc, l <- [refusal]]
 where
  say = phrase T.catalogue lang
  doc = dfAssemble Dead.doc req
  dead d =
    say
      "dead"
      [ R (key "name" d)
      , W (txt "verdict" d)
      , P (say (if int "whyCode" d == 1 then "why_unreach" else "why_unref") [])
      , P (trust (key "confidence" d))
      ]
  trust c = case c of
    Number 0 -> say "unvouched" []
    Number 1 -> say "vacuous" []
    Number 2 -> say "vouched" []
    _ -> plain ""
  aggregate a = say "aggregate" [R (key "name" a), W (txt "verdict" a)]
  advisory
    | key "unmentioned" doc == Null = []
    | bool "unmentioned_dropped" doc = [say "advisory_dropped" [N unmentionedCap]]
    | otherwise =
        let rows' = arr "unmentioned" doc
            by c = N (toInteger (length [() | a <- rows', txt "code" a == c]))
         in [say "advisory" [R (key "name" a), N (int "line" a), R (key "symbol" a), W (txt "code" a), W (txt "why" a)] | a <- rows']
              <> [say "advisory_census" ([N (count rows'), N (count (nub (map (key "name") rows')))] <> map by codeNames)]
              <> [say "advisory_cut" [N unmentionedCap] | bool "unmentioned_cut" doc]
  codeNames = words "public_unmentioned private_unmentioned restricted_unmentioned reexported_unmentioned"
  nDead = count (arr "dead" doc)
  tail' =
    [say "summary" [N (int "nodes" (key "counts" doc)), N (int "kept_edges" (key "counts" doc)), N nDead, N (count (arr "reported" doc)), N (int "unresolved_sites" doc)]]
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say "entry_hint" [N nDead, N (fact req "files")] | nDead >= 2, nDead * 2 >= fact req "files"]
  refusal = case key "degraded" doc of
    Null -> say "check_dead" [N nDead]
    d -> say "check_degraded" [fillOf d]
  count = toInteger . length

-- | The judgment failed: a dead file, or a degraded run.
failed :: Value -> Bool
failed doc = not (null (arr "dead" doc)) || key "degraded" doc /= Null

veto :: DocReq -> Bool
veto req = flag req "check" && failed (dfAssemble Dead.doc req)
