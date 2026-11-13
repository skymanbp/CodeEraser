-- | The `ce deadcode` console lines (plan v2.32 step 5), transcribed
-- from cli/src/graph/deadcode/report.rs and the `--check` lines of
-- cli/src/main_cmds.rs: each unreferenced file with its verdict, reason
-- and trust; the aggregates; the symbol advisory when the road was asked
-- (the dropped table in one line, else its rows, the census by code
-- and the producer's cut); the summary, the degraded reason, the
-- `entry_globs` hint when half the file tier or more is unreferenced
-- (two at least — `files` the file nodes); and under `--check` (the
-- `check` fact) the refusal on stderr. The veto: `--check` and the
-- judgment's fail — an unreferenced file, or a degraded run.
--
-- Under the grouping (plan v2.35, the dead-file amendment, 5A) the roots
-- print first, each once with the number of files that fall with it,
-- then the dependents, each once naming its roots (a list longer than
-- three shortened by a count; the document never shortens); the summary
-- counts the roots. When the core dropped the pairs the roots print
-- without a count, the dependents without roots, and one line says so
-- with the core's cap. The word is "unreferenced": the finding is that
-- no reference was found, not that the file is unused.
module CE.Graph.Lines (lines', veto) where

import CE.Document.Read
import CE.Graph.Cost (underCap, unmentionedCap)
import CE.Graph.Document (advisoryNames)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) (map dead ordered <> map aggregate (arr "reported" doc) <> advisory <> tail')
    <> [line 1 l | flag req "check", failed doc, l <- [refusal]]
 where
  rows' = arr "dead" doc
  counts = key "counts" doc
  told = key "roots" counts /= Null
  dropped = bool "under_dropped" doc
  ordered = if told then [d | d <- rows', bool "root" d] <> [d | d <- rows', not (bool "root" d)] else rows'
  dead d =
    say
      "dead"
      [ R (key "name" d)
      , W (txt "verdict" d)
      , P (say (if int "whyCode" d == 1 then "why_unreach" else "why_unref") [])
      , P (trust (key "confidence" d))
      , P (grouping d)
      ]
  grouping d
    | not told || dropped = plain ""
    | bool "root" d = say "falls" [N (M.findWithDefault 0 (show (key "name" d)) falls)]
    | otherwise = case arr "under" d of
        rs | length rs > 3 -> say "under_more" [P (join ", " (map (piece . R) (take 3 rs))), N (toInteger (length rs - 3))]
        rs -> say "under" [P (join ", " (map (piece . R) rs))]
  -- the dependents under each root, counted once over the pairs (a
  -- root's name is a reference object; its printed form is the key)
  falls = M.fromListWith (+) [(show r, 1 :: Integer) | x <- rows', r <- arr "under" x]
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
        let rowsA = arr "unmentioned" doc
            by c = N (toInteger (length [() | a <- rowsA, txt "code" a == c]))
         in [say "advisory" [R (key "name" a), N (int "line" a), R (key "symbol" a), W (txt "code" a), W (txt "why" a)] | a <- rowsA]
              <> [say "advisory_census" ([N (count rowsA), N (toInteger (S.size (S.fromList (map (key "name") rowsA))))] <> map by advisoryNames)]
              <> [say "advisory_cut" [N unmentionedCap] | bool "unmentioned_cut" doc]
  nDead = count rows'
  base = [N (int "nodes" counts), N (int "kept_edges" counts), N nDead]
  rest = [N (count (arr "reported" doc)), N (int "unresolved_sites" doc)]
  tail' =
    [if told then say "summary_roots" (base <> [N (int "roots" counts)] <> rest) else say "summary" (base <> rest)]
      <> [say "grouping_dropped" [N underCap] | told, dropped]
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say "entry_hint" [N nDead, N (fact req "files")] | nDead >= 2, nDead * 2 >= fact req "files"]
  refusal = case key "degraded" doc of
    Null -> say "check_dead" [N nDead]
    d -> say "check_degraded" [fillOf d]
  count = toInteger . length

-- | The judgment failed: an unreferenced file, or a degraded run.
failed :: Value -> Bool
failed doc = not (null (arr "dead" doc)) || key "degraded" doc /= Null

veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && failed doc
