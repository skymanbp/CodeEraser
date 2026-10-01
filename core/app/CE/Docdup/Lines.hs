-- | The `ce docdup` console lines (plan v2.32 step 5): the report
-- through the shared throat (cli/src/report.rs `emit` with the
-- templates of cli/src/docdup/judge/mod.rs `print`) — one line per
-- duplicate pair, each segment as `path:start-end kind`, then the
-- summary with the counters it does not name as a raw tail — and under
-- `--check` (the `check` fact) the refusal on stderr when any pair was
-- reported. The veto is that refusal.
module CE.Docdup.Lines (lines', veto) where

import CE.Document.Read
import qualified CE.Docdup.Document as Docdup
import qualified Data.Map.Strict as M

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  emitted say hit "dups" (words "dups segments judged jaccard_dups") doc
    <> [line 1 (say "check" [N (toInteger (length dups))]) | veto doc req]
 where
  dups = arr "dups" doc
  segs = Docdup.segments req
  hit d = [P (seg (key "a" d)), P (seg (key "b" d)), N (int "inter" d), N (int "union" d), N (int "verbatim" d)]
  seg r = case fromJSON (key "$" r) >>= \(c, s) -> if c == ("seg" :: String) then pure (M.lookup (s :: Integer) segs) else Error c of
    Success (Just (f, a, b, k)) -> say "seg" [R (ref "path" [f]), N a, N b, W k]
    _ -> plain "?"

-- | `--check` with a reported duplication.
veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && not (null (arr "dups" doc))
