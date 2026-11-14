-- | The `ce check` (and `ce baseline`) console lines (plan v2.32 step
-- 5), transcribed from cli/src/score/report.rs (gone at e35fde53): the score line (since
-- the extent law: the lines the findings are on, E of L with E to one
-- decimal, and each axis's own share per 1000 lines), the
-- ratchet line with the conditions that held, the collapse note when
-- something collapsed, the degraded line, and under `--roast` (the
-- `roast` fact) one line by score band against the effective scale.
-- An axis whose effective weight is 0 (the document's `weights`, a
-- judged run) prints its share marked "(off)": the axis does not count
-- (plan v2.35, the dead-file amendment); a held `dead_added` adds one
-- line with the blocking files' count and the command that lists them.
-- The veto is the verdict's fail bit (the face's own exit rule; a
-- judgment that never happened is the face's refusal, exit 2).
module CE.Score.Lines (lines', veto) where

import CE.Document.Read
import Data.List (intercalate)

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) $
    [ say "head" ([N (fact req "score"), N scale] <> extent <> [axis c | c <- [0 .. 6]] <> [N (count (arr "candidates" doc))])
    , say "ratchet" (map (N . count . (`arr` ratchet)) (words "added removed over toleranceDrawn") <> [P verdict])
    ]
      <> [say "dead_added" [N (count (arr "deadAdded" ratchet))] | "dead_added" `elem` failed]
      <> [say "note" [N (fact req "collapsed"), N (fact req "skippedSelf")] | fact req "collapsed" > 0 || fact req "skippedSelf" > 0]
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say roast [] | flag req "roast"]
 where
  ratchet = key "ratchet" doc
  -- the effective scale, 1000 when the knob is absent
  scale = case key "scoreScale" doc of
    Number n -> round n
    _ -> 1000
  -- the seven own shares by axis code, 0 where no row rode (a degraded run)
  shareOf c = maybe 0 id (lookup c [(c', p) | [c', p] <- rows req "axes"])
  -- the share, marked when the axis weighs 0 (no mark without the table)
  axis c
    | lookup c [(c', w) | [c', w] <- rows req "weights"] == Just 0 = P (say "axis_off" [N (shareOf c)])
    | otherwise = N (shareOf c)
  -- E and L, two holes, as the document states them (`extent.tenths`,
  -- the core's one rounding, CE.Verdict.Extent.tenths); "-" in both
  -- when no extent rode
  extent = case (key "tenths" (key "extent" doc), key "lines" (key "extent" doc)) of
    (Number t, Number len) -> let t' = round t :: Integer in [W (show (t' `quot` 10) <> "." <> show (t' `rem` 10)), N (round len)]
    _ -> [W "-", W "-"]
  failed = [n | v <- arr "failed" ratchet, Success n <- [fromJSON v]] :: [String]
  verdict
    | flag req "fail" = say "fail" [P (if null failed then plain "" else say "failed" [W (intercalate ", " failed)])]
    | otherwise = say "pass" []
  roast = case fact req "score" * 1000 `quot` max 1 scale of
    b | b >= 900 -> "roast_clean"
    b | b >= 750 -> "roast_slow"
    b | b >= 600 -> "roast_half"
    _ -> "roast_exorcist"
  count = toInteger . length

-- | The verdict's fail bit.
veto :: Value -> DocReq -> Bool
veto _ req = flag req "fail"
