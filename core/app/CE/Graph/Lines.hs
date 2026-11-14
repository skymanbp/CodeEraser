-- | The `ce deadcode` console lines (plan v2.32 step 5), transcribed
-- from cli/src/graph/deadcode/report.rs and the `--check` lines of
-- cli/src/main_cmds.rs: each unreferenced file with its verdict, reason
-- and trust; the aggregates; the symbol advisory when the road was asked
-- (the dropped table in one line, else its rows, the census by code
-- and the producer's cut); the summary, the degraded reason, the
-- `entry_globs` hint when half the file tier or more is unreferenced
-- (two at least — `files` the file nodes); and under `--check` (the
-- `check` fact) the refusal on stderr. The veto: `--check` and the
-- judgment's fail — a blocking file (a root of the unreferenced files
-- whose state the project blocks), or a degraded run (plan v2.35, 5B).
--
-- Under the grouping (plan v2.35, the dead-file amendment, 5A) the roots
-- print first, each once with the number of files that fall with it
-- (none said when none does),
-- then the dependents, each once naming its roots (a list longer than
-- three shortened by a count; the document never shortens); the summary
-- counts the roots. When the core dropped the pairs the roots print
-- without a count, the dependents without roots, and one line says so
-- with the core's cap. The word is "unreferenced": the finding is that
-- no reference was found, not that the file is unused. Every file's
-- state (5B), root or dependent, prints where the file is listed, and
-- only when a comparison point rode: without one every file is untold,
-- and a word on every row would tell nothing (the document keeps
-- `state` and its count); the summary then counts the files by state.
-- A blocking dependent is marked by its state word like a blocking root.
module CE.Graph.Lines (lines', veto) where

import CE.Document.Read
import CE.Graph.Cost (underCap, unmentionedCap)
import CE.Graph.Document (advisoryNames)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) (map dead ordered <> map aggregate (arr "reported" doc) <> advisoryLines say doc <> tail')
    <> [line 1 l | flag req "check", l <- refused]
    <> [line 0 l | flag req "check", null refused, nDead > 0, l <- [listed say doc]]
 where
  refused = refusal say doc
  rows' = arr "dead" doc
  counts = key "counts" doc
  told = key "roots" counts /= Null
  dropped = bool "under_dropped" doc
  ordered = if told then [d | d <- rows', bool "root" d] <> [d | d <- rows', not (bool "root" d)] else rows'
  dead d =
    say
      "dead"
      [ P (if compared doc && key "state" d /= Null then say "state_mark" [P (stateWord say (txt "state" d))] else plain "")
      , R (key "name" d)
      , W (txt "verdict" d)
      , P (say (if int "whyCode" d == 1 then "why_unreach" else "why_unref") [])
      , P (trust say (key "confidence" d))
      , P (grouping d)
      ]
  grouping d
    | not told || dropped = plain ""
    | bool "root" d = case M.findWithDefault 0 (key "name" d) falls of
        -- nothing falls with it: no suffix (early review G2)
        0 -> plain ""
        k -> say "falls" [N k]
    | otherwise = case arr "under" d of
        rs | length rs > 3 -> say "under_more" [P (join ", " (map (piece . R) (take 3 rs))), N (toInteger (length rs - 3))]
        rs -> say "under" [P (join ", " (map (piece . R) rs))]
  -- the dependents under each root, counted once over the pairs (a
  -- root's name is a reference object, keyed as itself: aeson's Value
  -- is ordered)
  falls = M.fromListWith (+) [(r, 1 :: Integer) | x <- rows', r <- arr "under" x]
  aggregate a = say "aggregate" [R (key "name" a), W (txt "verdict" a)]
  nDead = toInteger (length rows')
  base = [N (int "nodes" counts), N (int "kept_edges" counts), N nDead]
  rest = [N (toInteger (length (arr "reported" doc))), N (int "unresolved_sites" doc)]
  tail' =
    [summary]
      <> [say "unanswered" [blockWords say doc] | not (answered doc), not (flag req "check")]
      <> [say "grouping_dropped" [N underCap] | told, dropped]
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say "entry_hint" [N nDead, N (fact req "files")] | nDead >= 2, nDead * 2 >= fact req "files"]
  summary
    | compared doc = say "summary_states" (base <> map (N . (`int` counts)) (words "roots standing orphaned born") <> rest)
    | told = say "summary_roots" (base <> [N (int "roots" counts)] <> rest)
    | otherwise = say "summary" (base <> rest)

-- | A row's trust word, by its confidence code.
trust :: Say -> Value -> Piece
trust say c = case c of
  Number 0 -> say "unvouched" []
  Number 1 -> say "vacuous" []
  Number 2 -> say "vouched" []
  _ -> plain ""

-- | The symbol advisory when the road asked: the dropped table in one
-- line, else its rows, the census by code and the producer's cut.
advisoryLines :: Say -> Value -> [Piece]
advisoryLines say doc
  | key "unmentioned" doc == Null = []
  | bool "unmentioned_dropped" doc = [say "advisory_dropped" [N unmentionedCap]]
  | otherwise =
      [say "advisory" [R (key "name" a), N (int "line" a), R (key "symbol" a), W (txt "code" a), W (txt "why" a)] | a <- rowsA]
        <> [say "advisory_census" ([N (count rowsA), N (toInteger (S.size (S.fromList (map (key "name") rowsA))))] <> map by advisoryNames)]
        <> [say "advisory_cut" [N unmentionedCap] | bool "unmentioned_cut" doc]
 where
  rowsA = arr "unmentioned" doc
  by c = N (count [() | a <- rowsA, txt "code" a == c])
  count = toInteger . length

-- | Whether a comparison point rode: every file is untold exactly when
-- none did, so one told file, or the gate's comparison, says one did.
compared :: Value -> Bool
compared doc = (gate /= Null && key "comparison" gate /= Null) || any told (arr "dead" doc)
 where
  gate = key "gate" doc
  told d = key "state" d /= Null && txt "state" d /= "untold"

-- | The gate could be answered (or none was declared).
answered :: Value -> Bool
answered doc = key "gate" doc == Null || bool "answered" (key "gate" doc)

stateWord :: Say -> String -> Piece
stateWord say s = say ("st_" <> s) []

-- | The states the project blocks, by their words.
blockWords :: Say -> Value -> Fill
blockWords say doc = P (join ", " (map (stateWord say . unString) (arr "block" (key "gate" doc))))
 where
  unString v = case fromJSON v of
    Success t -> t
    _ -> ""

-- | Under `--check` (stderr): the degraded run, the unanswered gate, or
-- the blocking files by name (the first eight; the rows above list them
-- all) with the states that blocked them.
refusal :: Say -> Value -> [Piece]
refusal say doc = case key "degraded" doc of
  Null
    | not (answered doc) -> [say "check_unanswered" [blockWords say doc]]
    | length blocking > 8 -> [say "check_blocked_more" [N (count blocking), blockedStates, names (take 8 blocking), N (count blocking - 8)]]
    | not (null blocking) -> [say "check_blocked" [N (count blocking), blockedStates, names blocking]]
    | otherwise -> []
  d -> [say "check_degraded" [fillOf d]]
 where
  blocking = [d | d <- arr "dead" doc, bool "blocking" d]
  blockedStates = P (join ", " (map (stateWord say) (S.toList (S.fromList [txt "state" d | d <- blocking]))))
  names = P . join ", " . map (piece . R . key "name")
  count = toInteger . length

-- | A passing check with files listed: listed, not blocked, and how a
-- project turns blocking on (or which states it already blocks).
listed :: Say -> Value -> Piece
listed say doc
  | gate /= Null && not (null (arr "block" gate)) = say "check_listed_gated" [N nDead, blockWords say doc]
  | otherwise = say "check_listed" [N nDead]
 where
  gate = key "gate" doc
  nDead = toInteger (length (arr "dead" doc))

-- | The judgment failed: a degraded run, or a blocking file (plan v2.35,
-- the dead-file amendment: an unreferenced file in no blocked state is
-- listed and fails nothing by itself). An unanswered gate is the face's
-- own refusal (exit 2), never this bit.
failed :: Value -> Bool
failed doc = key "degraded" doc /= Null || any (bool "blocking") (arr "dead" doc)

veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && failed doc
