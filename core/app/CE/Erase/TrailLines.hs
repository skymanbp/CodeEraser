-- | The `ce erase --log` console lines (plan v2.32 step 5), transcribed
-- from cli/src/erase/log.rs `print`: one line per record (its UTC
-- stamp, class, target, provenance and plan hash), every refused line
-- by number, one summary — the trail's size, or that no trail exists
-- yet. The veto: a refused line (the audit file's evidence is broken).
module CE.Erase.TrailLines (lines', utcStamp, veto) where

import CE.Document.Read
import CE.Erase.Document (logRel)
import CE.Erase.Lines (spanText)
import Text.Printf (printf)

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ =
  map (line 0) $
    map record (arr "rows" doc)
      <> [say "unreadable" [N (int "line" u), R (key "why" u)] | u <- arr "unreadable" doc]
      <> [summary]
 where
  counts = key "counts" doc
  record r =
    say
      "record"
      [ W (utcStamp (int "ts_ms" r))
      , W (txt "class" r)
      , R (key "path" r)
      , W (spanText (key "span" r))
      , R (key "provenance" r)
      , R (key "plan" r)
      ]
  summary
    | bool "present" doc = say "log" [N (int "rows" counts), W logRel, N (int "unreadable" counts)]
    | otherwise = say "no_log" [W logRel]

-- | `YYYY-MM-DDTHH:MM:SSZ` from epoch milliseconds — the proleptic
-- Gregorian civil date of the day count (Howard Hinnant's
-- `civil_from_days`, integer-exact, as cli/src/erase/log.rs
-- `utc_stamp` computes it).
utcStamp :: Integer -> String
utcStamp ms = printf "%04d-%02d-%02dT%02d:%02d:%02dZ" y m d (rest `div` 3600) (rest `mod` 3600 `div` 60) (rest `mod` 60)
 where
  secs = ms `div` 1000
  rest = secs `mod` 86400
  z = secs `div` 86400 + 719468
  (era, doe) = z `divMod` 146097
  yoe = (doe - doe `div` 1460 + doe `div` 36524 - doe `div` 146096) `div` 365
  doy = doe - (365 * yoe + yoe `div` 4 - yoe `div` 100)
  mp = (5 * doy + 2) `div` 153
  d = doy - (153 * mp + 2) `div` 5 + 1
  m = if mp < 10 then mp + 3 else mp - 9
  y = yoe + era * 400 + (if m <= 2 then 1 else 0)

-- | A refused line fails the read.
veto :: Value -> DocReq -> Bool
veto doc _ = not (null (arr "unreadable" doc))
