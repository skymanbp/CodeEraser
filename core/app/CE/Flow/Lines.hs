-- | The `ce flow` console lines (plan v2.32 step 5), transcribed from
-- cli/src/flow_report/console.rs: the degraded reason first, one line
-- per listed finding (`path:line  unit  kind  var`, an advisory one
-- marked), the units left unjudged with their reasons, one counts
-- line. The veto is `--check`'s under the deny tier: a judged finding
-- fails (a degraded document is the face's own refusal, exit 2).
module CE.Flow.Lines (lines', veto) where

import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ =
  map (line 0) $
    [say "degraded" [R d] | let d = key "degraded" doc, d /= Null]
      <> map finding (arr "findings" doc)
      <> [say "unjudged" [R (key "path" x), R (key "unit" x), R (key "reason" x)] | x <- arr "refused" doc]
      <> [say "counts" [N (c k) | k <- words "units findings judged shown dynamicUnits refused"]]
 where
  c k = int k (key "counts" doc)
  finding f =
    say
      "finding"
      [ R (key "path" f)
      , N (int "line" f)
      , R (key "unit" f)
      , fillOf (key "kind" f)
      , if key "var" f == Null then W "-" else R (key "var" f)
      , P (if bool "judged" f then plain "" else say "advisory" [])
      ]

-- | `--check` under the deny tier fails on a judged finding.
veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && flag req "deny" && int "judged" (key "counts" doc) > 0
