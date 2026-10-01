-- | The `ce graph --mentions` console lines (plan v2.32 step 5),
-- transcribed from cli/src/mention/face.rs: the universe, what was
-- skipped, this run's writes (with the full-rescan suffix when the
-- revision moved), the judged files outside the universe, the bundler
-- runs, then one census line per language in name order. No judgment,
-- no veto.
module CE.Mention.Lines (lines') where

import CE.Document.Contract
import CE.Text
import qualified CE.Text.Mentions as T
import Data.List (sortOn)

lines' :: Lang -> DocReq -> [Line]
lines' lang req =
  map (line 0) $
    [ say "universe" (map f (words "universe sources rows capped mention_rev"))
    , say "skipped" (map f (words "skipped.oversize skipped.binary skipped.signed skipped.walk_errors"))
    , say "run" (map f (words "run.refreshed run.removed run.clipped run.starved") <> [P (if flag req "run.rescanned" then say "rescan" [] else plain "")])
    , say "outside" (map f (words "outside.oversize outside.binary outside.nested outside.ignored"))
    , say "dist" [f "dist_js_dedup_runs"]
    ]
      <> [say "rates" [W (langName c), N da, N de, N ua, N ue, N other, N saved, N fold, N self] | [c, da, de, ua, ue, other, fold, self, saved] <- sortOn byName (rows req "rates")]
 where
  say = phrase T.catalogue lang
  f k = N (fact req k)
  byName r = langName (head' r)
  head' r = case r of
    c : _ -> c
    [] -> -1
