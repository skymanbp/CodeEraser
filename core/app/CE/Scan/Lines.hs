-- | The `ce scan` console lines (plan v2.32 step 5), transcribed from
-- cli/src/scan/report.rs `print_console`: one line per finding (its
-- tag, place, rule, value, the line it crossed and its subject), then
-- the summary — with ` -> FAIL` and the held conditions when any held.
-- The veto is the core's fail bit: a held condition (`ce scan` exits 1).
module CE.Scan.Lines (lines', veto) where

import CE.Document.Read
import Data.List (intercalate)

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ =
  map (line 0) $
    map finding (arr "findings" doc)
      <> [say "summary" (map (N . (`int` summary)) (words "files functions warns fails") <> [P verdict])]
 where
  summary = key "summary" doc
  finding f =
    say
      "finding"
      [ W (if txt "level" f == "fail" then "FAIL" else "warn")
      , R (key "file" f)
      , N (int "line" f)
      , W (txt "rule" f)
      , N (int "value" f)
      , N (int "threshold" f)
      , R (key "subject" f)
      ]
  failed = [n | v <- arr "failed" doc, Success n <- [fromJSON v]] :: [String]
  verdict
    | null failed = plain ""
    | otherwise = say "fail" [P (say "failed" [W (intercalate ", " failed)])]

-- | A held condition fails the scan.
veto :: Value -> DocReq -> Bool
veto doc _ = not (null (arr "failed" doc))
