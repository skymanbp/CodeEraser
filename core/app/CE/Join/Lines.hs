-- | The `ce join` console lines (plan v2.32 step 5), transcribed from
-- cli/src/join/report.rs: one line per file pair with its three legs
-- and the lattice's verdict, one per unit pair with its family's
-- metric and churn, the degraded leg, the window summary.
-- Report-only: no veto.
module CE.Join.Lines (lines') where

import CE.Document.Read
import qualified CE.Join.Document as Join
import qualified CE.Text.Join as T

lines' :: Lang -> DocReq -> [Line]
lines' lang req =
  map (line 0) $
    map file (arr "files" doc)
      <> map unit (arr "units" doc)
      <> [say "degraded" [fillOf d] | let d = key "degraded" doc, d /= Null]
      <> [say "summary" [N (int "days" doc), N (count (arr "files" doc)), N (count (arr "units" doc)), N (int "commits" doc)]]
 where
  say = phrase T.catalogue lang
  doc = dfAssemble Join.doc req
  count = toInteger . length
  churn k v = [N (int "appended" (key k v)), N (int "rewrote" (key k v))]
  file f =
    say "file" $
      [R (key "a" f), R (key "b" f), N (int "blocks" f), N (int "tokens" f), N (int "near_miss" f), P (pos (key "graph_a" f)), P (pos (key "graph_b" f))]
        <> churn "churn_a" f
        <> churn "churn_b" f
        <> [W (case key "cochange" f of Number n -> show (round n :: Integer); _ -> "-"), P (verdict f)]
  verdict f
    | key "verdict" f /= Null && key "severity" f /= Null && key "confidence" f /= Null =
        say "verdict" [W (txt "verdict" f), N (int "severity" f), N (int "confidence" f)]
    | key "a" f == key "b" f = say "self_pair" []
    | otherwise = say "unjudged" []
  pos v = case fromJSON v of
    Success [i, o, s, z, r] -> say "pos" (map N [i, o, s, z, r])
    _ -> say "pos_null" []
  unit u =
    say "unit" $
      side (key "a" u) <> side (key "b" u) <> [P (sim u)] <> churn "churn_a" u <> churn "churn_b" u
  side s = [R (key "path" s), R (key "key" s), N (int "nth" s)]
  sim u
    | txt "kind" u == "t1t2" = say "tokens" [N (int "tokens" u)]
    | otherwise = say "t3" [N (int "ted" u), N (int "n1" u), N (int "n2" u)]
