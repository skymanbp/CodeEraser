-- | The `ce trend` console lines (plan v2.32 step 5), transcribed from
-- cli/src/trend/report.rs and the veto of cli/src/main_judge.rs
-- `trend_cmd`: one line per point (its short commit, stamp, score over
-- scale, axes), every refused commit, the verdict with the slope in
-- per mille of the scale per day and ` -> FAIL` when the judgment
-- fails, the cliff and the decline run, the window; then on stderr the
-- veto's sentence — the decline past the declared floor first, else
-- the first commit that refused to measure. The veto: the judgment's
-- fail bit, or a refused commit.
module CE.Trend.Lines (lines', veto) where

import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ =
  map (line 0) (zipWith point [0 ..] (arr "rows" doc) <> map failed refusals <> [verdict] <> shape <> [window])
    <> [line 1 (say "check" [P why]) | Just why <- [maybe refused Just declined]]
 where
  j = key "judgment" doc
  refusals = arr "failed" doc
  short i = R (ref "short" [i])
  pm v = W (fixed 1 v 1000)
  -- the points are one per place (the contract), so a point's place
  -- is its commit's
  point i p = say "point" [short i, N (int "ts" p), N (int "score" p), N (int "scale" p), W (unwords [show c <> ":" <> show v | [c, v] <- axes p])]
  axes p = case fromJSON (key "axes" p) of
    Success xs -> xs
    _ -> [] :: [[Integer]]
  failed f = say "failed" (map R (ref2 f))
  ref2 f = case fromJSON f of
    Success xs -> xs
    _ -> [] :: [Value]
  slope = case key "slopeMicroPerDay" j of
    Number s -> Just (round s)
    _ -> Nothing
  verdict = say "verdict" [P (say word []), P (maybe (plain "") (\s -> say "slope" [pm s]) slope), W (if bool "fail" j then " -> FAIL" else "")]
  word = case key "verdict" j of
    Number 0 -> "improving"
    Number 1 -> "flat"
    Number 2 -> "degrading"
    _ -> "unjudged"
  shape =
    [say "cliff" [pm d, short i] | Success [i, d] <- [fromJSON (key "cliff" j)]]
      <> [say "decline" [N n, short i] | Success [i, n] <- [fromJSON (key "declineRun" j)]]
  window = say "window" [N (int "window" doc), N (toInteger (length (arr "rows" doc))), N (int "pending" doc)]
  floor' = case [v | Success [1, v] <- map fromJSON (arr "knobs" j)] of
    v : _ -> pm v
    [] -> W "?"
  declined = if bool "fail" j then Just (say "declined" [maybe (W "?") (pm . negate) slope, floor']) else Nothing
  refused = case refusals of
    f : _ -> Just (say "refused" ([N (toInteger (length refusals)), N (int "window" doc)] <> map R (ref2 f)))
    [] -> Nothing

-- | The judgment fails, or a commit refused to measure.
veto :: Value -> DocReq -> Bool
veto doc _ = bool "fail" (key "judgment" doc) || not (null (arr "failed" doc))
