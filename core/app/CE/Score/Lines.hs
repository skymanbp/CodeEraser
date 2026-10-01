-- | The `ce check` (and `ce baseline`) console lines (plan v2.32 step
-- 5), transcribed from cli/src/score/report.rs: the score line, the
-- ratchet line with the conditions that held, the collapse note when
-- something collapsed, the degraded line, and under `--roast` (the
-- `roast` fact) one line by score band against the effective scale.
-- The veto is the verdict's fail bit (the face's own exit rule; a
-- judgment that never happened is the face's refusal, exit 2).
module CE.Score.Lines (lines', veto) where

import CE.Document.Read
import qualified CE.Score.Document as Check
import qualified CE.Text.Check as T
import Data.List (intercalate)

lines' :: Lang -> DocReq -> [Line]
lines' lang req =
  map (line 0) $
    [ say "head" [N (fact req "score"), N scale, W axes, N (count (arr "candidates" doc))]
    , say "ratchet" (map (N . count . (`arr` ratchet)) (words "added removed over toleranceDrawn") <> [P verdict])
    ]
      <> [say "note" [N (fact req "collapsed"), N (fact req "skippedSelf")] | fact req "collapsed" > 0 || fact req "skippedSelf" > 0]
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say roast [] | flag req "roast"]
 where
  say = phrase T.catalogue lang
  doc = dfAssemble Check.doc req
  ratchet = key "ratchet" doc
  -- the effective scale, 1000 when the knob is absent
  scale = case key "scoreScale" doc of
    Number n -> round n
    _ -> 1000
  axes = unwords [show c <> ":" <> show p | [c, p] <- rows req "axes"]
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
veto :: DocReq -> Bool
veto req = flag req "fail"
