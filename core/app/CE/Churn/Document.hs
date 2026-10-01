-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce churn` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/churn/report.rs
-- `report_json`): the window's commits, the ledger's two sums and the
-- lines it added, how many survive at HEAD and how many churned, the
-- co-change pairs, the commits too large to pair and the declared
-- submodules whose history is not this repository's. No judgment: the
-- measuring side reads git and sends the sums, the surviving count and
-- the pairs; the arithmetic, the field set and the console's display
-- cut are stated here, and the pairing cap the measurement uses rides
-- in the catalogue.
module CE.Churn.Document (cochangeFileCap, displayCut, doc) where

import CE.Document.Contract
import Data.Aeson (Value, object, (.=))

doc :: DocFamily
doc = docFamily "churn" "ce.churn-report/0.2.0" statement (const Nothing) assemble ["cochangeFileCap" .= cochangeFileCap, "displayCut" .= displayCut]

-- | Commits changing more files than this are left out of the pair
-- count (quadratic) and counted instead (cli/src/churn/report.rs
-- `COCHANGE_FILE_CAP`); the console names the number beside the count.
cochangeFileCap :: Integer
cochangeFileCap = 20

-- | The co-change pairs the console prints; the rest are counted.
displayCut :: Int
displayCut = 20

-- | `days` the window; `appended` / `rewrote` the ledger's two sums,
-- `surviving` the added lines alive at HEAD, `skipped` the commits past
-- the cap; a co-change row [a, b, commits], strongest first; each
-- submodule is a reference by its place.
statement :: String
statement =
  "range paths\nrange submodules\n\
  \fact days judged\nfact commits judged\nfact appended judged\nfact rewrote judged\n\
  \fact surviving judged\nfact skipped judged\n\
  \rows cochange 3 judged paths paths -\n\
  \ref path paths\nref submodule submodules\n"

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "commits" .= fact req "commits"
    , "append_lines" .= fact req "appended"
    , "rewrite_lines" .= fact req "rewrote"
    , "added_in_window" .= added
    , "surviving" .= fact req "surviving"
    , "churned" .= max 0 (added - fact req "surviving")
    , "cochange" .= [object ["a" .= ref "path" [a], "b" .= ref "path" [b], "count" .= n] | [a, b, n] <- rows req "cochange"]
    , "skipped_large_commits" .= fact req "skipped"
    , "submodules_without_file_history" .= [ref "submodule" [s] | s <- [0 .. range req "submodules" - 1]]
    ]
 where
  added = fact req "appended" + fact req "rewrote"
