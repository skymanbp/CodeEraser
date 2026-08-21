-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | audit.request handler (M9 batch 7 slice 7): the tenth judgment
-- family — the session/commit duplication VERDICT behind the Stop
-- audit and `ce precommit`. Rust measures set membership (is each
-- side of each clone block among the session's changed files) and
-- sends one [aTouched, bTouched] bit row per block; this family
-- answers which rows convict and whether the session may end. Paths
-- never cross the wire (§5.9.2) — row index is identity and Rust
-- re-labels on return. Knobless like erase/1: zero tolerance is not
-- tunable — a knob that tolerated one touched duplicate would be a
-- licence to stack.
module CE.Audit (respond) where

import CE.Audit.Cost (auditBlockCap, dupTolerance, judgeRow)
import CE.Wire (RowsReq (..), knoblessRows)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | The shared knobless cascade with this family's bindings
-- (CE.Wire.knoblessRows): every row [aTouched, bTouched], both
-- strict booleans — a count smuggled in as a bit would judge
-- as touched and must refuse by name instead.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = knoblessRows "audit" auditBlockCap rowShape (degraded proto) (judged proto)

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [a, b]
    | any (`notElem` [0, 1]) [a, b] -> Just (label <> "touched bit not a boolean")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [aTouched,bTouched])")
 where
  label = "row " <> show i <> ": "

judged :: String -> RowsReq -> B8.ByteString
judged proto req = reply proto req dups False
 where
  dups = [i | (i, row) <- zip [0 :: Integer ..] (rowsOf req), judgeRow row]

-- | Over-cap: a complete degraded reply that FAILS with an empty
-- conviction table — an index the core refused to judge licenses
-- nothing, and the CLI's fail-open discipline turns this into a
-- VISIBLE degraded skip, never a silent pass.
degraded :: String -> RowsReq -> B8.ByteString
degraded proto req = reply proto req [] True

-- | The audit.result object: the convicted row indices in request
-- order, and fail = the zero-tolerance verdict itself (more
-- convictions than dupTolerance, or a degraded reply).
reply :: String -> RowsReq -> [Integer] -> Bool -> B8.ByteString
reply proto req dups isDegraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("audit.result" :: String)
    , "id" .= rowsId req
    , "dups" .= dups
    , "counts"
        .= object
          [ "rows" .= length (rowsOf req)
          , "dups" .= length dups
          ]
    , "fail" .= (isDegraded || toInteger (length dups) > dupTolerance)
    , "degraded" .= isDegraded
    ]
      <> ["reason" .= ("audit_too_large" :: String) | isDegraded]
