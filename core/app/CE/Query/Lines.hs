-- | The `ce query` / `ce rules` console lines (plan v2.32 step 5),
-- transcribed from cli/src/query/console.rs: the errors first (a
-- program with an error answers nothing), the degraded reason, then
-- each goal — a question with its answers, an assertion with its
-- witnesses — every proof row indented by its depth in the
-- derivation, and one counts line; the rules face says first when no
-- rules file was found. The veto is the rules face's: a judged program
-- whose assertions found a violation (an unjudged one is the face's
-- own refusal, exit 2).
module CE.Query.Lines (lines', veto) where

import CE.Document.Read

lines' :: Bool -> Say -> Value -> DocReq -> [Line]
lines' rules say doc req =
  map (line 0) $
    [say "no_rules_file" [] | rules, fact req "rulesFile" == 0]
      <> [say "error" [fillOf (key "at" e), fillOf (key "what" e)] | e <- arr "errors" doc]
      <> [say "degraded" [R d] | let d = key "degraded" doc, d /= Null]
      <> (if judged doc then concatMap goal (arr "goals" doc) <> [counts] else [])
 where
  goal g =
    let mine = [a | a <- arr "answers" doc, int "goal" a == int "goal" g]
        cols = join ", " (map (piece . R) (arr "columns" g))
        name = if key "name" g == Null then W "" else R (key "name" g)
        head'
          | txt "kind" g /= "assert" = say "question" [P cols, N (count mine)]
          | null mine = say "assert_ok" [name, P cols]
          | otherwise = say "assert_violations" [name, P cols, N (count mine)]
     in head' : concat (zipWith (answered (int "goal" g)) [0 ..] mine)
  answered g i a = say "answer" [P (join "  " (map (piece . fillOf) (arr "values" a)))] : chain g i
  chain g i =
    let mine = [p | p <- arr "proof" doc, int "goal" p == g, int "answer" p == i]
        -- each node under its parent; a parent chain that loops (no
        -- core answers one) stops after every row was walked once
        depth node = walk node (0 :: Int)
        walk at d = case [p | p <- mine, int "node" p == at] of
          p : _ | int "parent" p >= 0, d <= length mine -> walk (int "parent" p) (d + 1)
          _ -> d
     in [ say "proof" [W (concat (replicate (depth (int "node" p) + 1) "  ")), fillOf (key "pred" p), P (join ", " (map (piece . fillOf) (arr "args" p))), P (by p)]
        | p <- mine
        , int "parent" p >= 0
        ]
  by p = if int "rule" p < 0 then say "by_fact" [] else say "by_clause" [N (int "rule" p)]
  c k = int k (key "counts" doc)
  counts = say "counts" [N (c "rules"), N (c "facts"), N (c "derived"), N (c "proofNodes"), P truncated]
  truncated = if c "proofTruncated" > 0 then say "truncated" [N (c "proofTruncated")] else plain ""
  count = toInteger . length

-- | A judged program: no error, no degraded reason.
judged :: Value -> Bool
judged doc = null (arr "errors" doc) && key "degraded" doc == Null

-- | The rules face's exit 1: judged, and a violation found.
veto :: Value -> DocReq -> Bool
veto doc req = judged doc && fact req "violations" > 0
