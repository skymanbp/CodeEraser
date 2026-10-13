-- | The `ce erase --log` console lines (plan v2.32 step 5), transcribed
-- from cli/src/erase/log.rs `print`: one line per record (the stamp
-- the document states, class, target, provenance and plan hash), every refused line
-- by number, one summary — the trail's size, or that no trail exists
-- yet. The veto: a refused line (the audit file's evidence is broken).
module CE.Erase.TrailLines (lines', veto) where

import CE.Document.Read
import CE.Erase.Document (logRel)
import CE.Erase.Lines (spanText)

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
      [ W (txt "stamp" r)
      , W (txt "class" r)
      , R (key "path" r)
      , W (spanText (key "span" r))
      , R (key "provenance" r)
      , R (key "plan" r)
      ]
  summary
    | bool "present" doc = say "log" [N (int "rows" counts), W logRel, N (int "unreadable" counts)]
    | otherwise = say "no_log" [W logRel]

-- | A refused line fails the read.
veto :: Value -> DocReq -> Bool
veto doc _ = not (null (arr "unreadable" doc))
