-- | The `ce graph --sites` console lines (plan v2.32 step 5),
-- transcribed from cli/src/graph/mod.rs `print_counts`: the total over
-- the files that hold a site, then one row per (language, kind) in
-- name order, the two names padded to 10 and 12 columns. No judgment,
-- no veto.
module CE.Graph.SitesLines (lines') where

import CE.Document.Contract
import CE.Document.Read (Say)
import CE.Lang (siteKinds)
import CE.Text
import Data.Aeson (Value)
import qualified Data.Map.Strict as M

lines' :: Say -> Value -> DocReq -> [Line]
lines' say _ req =
  map (line 0) $
    say "total" [N (toInteger (length sites)), N (toInteger (M.size (M.fromList [(f, ()) | _ : f : _ <- sites])))]
      : [say "count" [W (leftIn 10 l), W (leftIn 12 k), N n] | ((l, k), n) <- M.toAscList counted']
 where
  sites = rows req "sites"
  langs = M.fromList [(f, l) | [f, l] <- rows req "langs"]
  counted' = M.fromListWith (+) [((langName (M.findWithDefault (-1) f langs), kind k), 1 :: Integer) | _ : f : k : _ <- sites]
  kind k = concat (take 1 (drop (fromInteger k) siteKinds))
