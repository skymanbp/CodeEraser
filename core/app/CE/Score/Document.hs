-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce check` document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/score/report.rs
-- `report_json`, the outcome in cli/src/score/model.rs): the score
-- and its scale, the axes, the floor the run was armed with, the
-- candidate and severity rows, the ratchet with the conditions that
-- held, the measuring side's counts and the degraded reason. `ce
-- baseline` prints the same document. Every row is the verdict
-- reply's, sent back; the candidates' two file columns stay integers
-- (the document never carried their paths). No repository string
-- rides this document except the reason the measuring side holds when
-- the judgment never happened.
module CE.Score.Document (doc, failNames) where

import CE.Document.Contract
import CE.Verdict (degradedCondition)
import CE.Verdict.Faces (failConditions)
import CE.Verdict.Ratchet (Ratcheted (..))
import Data.Aeson (Value, object, (.=))
import Data.Foldable (asum)

doc :: DocFamily
doc = docFamily "check" schemaId statement checked assemble []

schemaId :: String
schemaId = "ce.check-report/0.5.0"

-- | The request: `files` the verdict universe (the candidates'
-- columns), the scale / floor / reason tables at most one row each
-- (null when empty), `failed` the held conditions by code, `dropped`
-- the provenance rows — present in the document exactly when
-- `droppedRode` is 1.
statement :: String
statement =
  "range files\nrange why\n\
  \fact score judged\nfact fail judged\nfact droppedRode judged\n\
  \fact simPairs judged\nfact members judged\nfact collapsed judged\nfact skippedSelf judged\n\
  \rows scale 1 judged -\nrows floor 1 judged -\nrows reason 1 judged -\n\
  \rows axes 2 judged - -\nrows candidates 6 judged files files - - - -\n\
  \rows joinSeverity 2 judged - -\nrows added 1 judged -\nrows removed 1 judged -\n\
  \rows over 4 judged - - - -\nrows toleranceDrawn 3 judged - - -\n\
  \rows failed 1 judged -\nrows dropped 3 judged - - -\n\
  \ref why why\n"

-- | The ratchet's fail conditions by code: the six the verdict face
-- names, in its order (read off CE.Verdict.Faces.failConditions with
-- every condition unheld — only the names are taken), then the
-- degraded reply's own (CE.Verdict). One spelling each, in the
-- modules that answer them.
failNames :: [String]
failNames = map fst (failConditions (Ratcheted [] [] [] [] [] [] []) False False False) <> [degradedCondition]

-- | The nullable tables one row at most, every code inside its table,
-- the two bits 0 / 1, and no dropped row unless the register rode.
checked :: DocReq -> Maybe String
checked req =
  asum
    [ asum [single req t | t <- ["scale", "floor", "reason"]]
    , codes req "reason" 0 0 (toInteger (length coreReasons) - 1)
    , codes req "failed" 0 0 (toInteger (length failNames) - 1)
    , asum [Just ("facts: " <> k <> " is not 0 or 1") | k <- ["fail", "droppedRode"], fact req k > 1]
    , if flag req "droppedRode" || null (rows req "dropped") then Nothing else Just "dropped: rows without the register"
    ]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= schemaId
    , "score" .= fact req "score"
    , "scoreScale" .= optional req "scale"
    , "axes" .= rows req "axes"
    , "floor" .= optional req "floor"
    , "candidates" .= rows req "candidates"
    , "joinSeverity" .= rows req "joinSeverity"
    , "ratchet" .= object (ratchet <> ["dropped" .= rows req "dropped" | flag req "droppedRode"])
    , "counts" .= counted countKeys (range req "files" : map (fact req) (drop 1 countKeys))
    , "degraded" .= degradedOf req ["reason"]
    ]
 where
  countKeys = words "files simPairs members collapsed skippedSelf"
  ratchet =
    [ "added" .= concat (rows req "added")
    , "removed" .= concat (rows req "removed")
    , "over" .= rows req "over"
    , "toleranceDrawn" .= rows req "toleranceDrawn"
    , "fail" .= flag req "fail"
    , "failed" .= [spelled failNames c | [c] <- rows req "failed"]
    ]
