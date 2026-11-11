-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce churn` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side until step 5's R0 pilot (the
-- measuring side now sends the integers, cli/src/churn/report.rs
-- `answer`): the window's commits, the ledger's two sums and the
-- lines it added, how many survive at HEAD and how many churned, the
-- lines blame gives the window beyond the ledger, the co-change pairs,
-- the commits too large to pair and the declared submodules whose
-- history is not this repository's. No judgment: the measuring side
-- reads git and sends the sums, the surviving count and the pairs; the arithmetic, the field set and the console's display
-- cut are stated here, and the pairing cap the measurement uses rides
-- in the catalogue.
module CE.Churn.Document (cochangeFileCap, displayCut, doc) where

import CE.Document.Contract
import Data.Aeson (Value, object, (.=))

doc :: DocFamily
doc = docFamily "churn" schemaId statement survivors assemble ["cochangeFileCap" .= cochangeFileCap, "displayCut" .= displayCut]

-- | Every survivor is an added line of the same window: the measuring
-- side counts a HEAD line only when its first-parent blame names a
-- window commit, over the files the ledger attributed, and caps each
-- (commit, path) at the lines the ledger added there — the rest is
-- `beyond_ledger` (cli/src/churn/survival.rs; v2.35 V1). More
-- survivors than added lines is a measurement that broke that rule,
-- refused by name rather than clamped to a churn of 0.
survivors :: DocReq -> Maybe String
survivors req
  | fact req "surviving" > added req = Just "facts: surviving exceeds the window's added lines"
  | otherwise = Nothing

-- | The lines the window added: the ledger's two sums.
added :: DocReq -> Integer
added req = fact req "appended" + fact req "rewrote"

schemaId :: String
schemaId = "ce.churn-report/0.3.0"

-- | Commits changing more files than this are left out of the pair
-- count (quadratic) and counted instead; the measurement reads it off
-- the catalogue entry (cli/src/churn/mod.rs) and the console names the
-- number beside the count.
cochangeFileCap :: Integer
cochangeFileCap = 20

-- | The co-change pairs the console prints; the rest are counted.
displayCut :: Int
displayCut = 20

-- | `days` the window; `appended` / `rewrote` the ledger's two sums,
-- `surviving` the added lines alive at HEAD, `beyond_ledger` the lines
-- blame gives a window commit past the ledger's added lines for that
-- commit and path (a line end, a final newline, another diff
-- alignment), `skipped` the commits past the cap; a co-change row
-- [a, b, commits], strongest first; each submodule is a reference by
-- its place.
statement :: String
statement =
  "range paths\nrange submodules\n\
  \fact days judged\nfact commits judged\nfact appended judged\nfact rewrote judged\n\
  \fact surviving judged\nfact beyond_ledger judged\nfact skipped judged\n\
  \rows cochange 3 judged paths paths -\n\
  \ref path paths\nref submodule submodules\n"

assemble :: DocReq -> Value
assemble req =
  object
    [ "commits" .= fact req "commits"
    , "append_lines" .= fact req "appended"
    , "rewrite_lines" .= fact req "rewrote"
    , "added_in_window" .= added req
    , "surviving" .= fact req "surviving"
    , "churned" .= (added req - fact req "surviving")
    , "beyond_ledger" .= fact req "beyond_ledger"
    , "cochange" .= [object ["a" .= ref "path" [a], "b" .= ref "path" [b], "count" .= n] | [a, b, n] <- rows req "cochange"]
    , "skipped_large_commits" .= fact req "skipped"
    , "submodules_without_file_history" .= [ref "submodule" [s] | s <- [0 .. range req "submodules" - 1]]
    ]
