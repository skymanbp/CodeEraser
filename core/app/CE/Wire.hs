-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The judgment-family wire skeleton (ADR-008 P3 ratchet
-- repayment: the sixth family recloned the respond cascade and the
-- ascending checker for the third time — the repo's own dedup gate
-- caught its author a tenth time, so the skeleton itself becomes
-- the single authority). A family says WHAT — its caps, offences
-- and replies — as data; this module owns the HOW of answering one
-- request line. CE.Verdict keeps its own cascade: its parsed
-- baseline threads through cap AND offence, a shape this skeleton
-- deliberately does not grow to cover.
-- The family modules import the skeleton from here (plan v2.33 step
-- 11a): the reply envelope (CE.Wire.Result), the retired keys
-- (CE.Wire.Retired) and the row vocabulary (CE.Wire.Rows) ride out with
-- it; only the family names are imported on their own, qualified
-- (CE.Wire.Families as F).
module CE.Wire
  ( Answered
  , Family (..)
  , FamilyName (..)
  , Respond
  , RowsReq (..)
  , Rulepack (..)
  , applyRows
  , family
  , knobbedRows
  , knoblessRows
  , pick
  , respondWith
  , rowsFamily
  , tableCap
  , tabled
  , module CE.Wire.Result
  , module CE.Wire.Retired
  , module CE.Wire.Rows
  ) where

import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import CE.Wire.Families (FamilyName (..))
import CE.Wire.Result
import CE.Wire.Retired
import CE.Wire.Rows
import Data.Foldable (asum)

-- | A family's answer to one request line: the reply line, or the
-- refusal the protocol turns into an error reply (the id when one
-- decoded, the error code, the message).
type Answered = Either (Maybe Value, String, String) B8.ByteString

-- | A family's responder: the protocol version it answers under, the
-- request line, the answer. Every family's `respond` has this type,
-- and the dispatch table (CE.Protocol) holds one per family.
type Respond = String -> B8.ByteString -> Answered

-- | The [[Integer]]-rows request the table families share: id, the
-- fact rows, and the optional side tables — Trend/Erase read knobs,
-- Scan reads grades and naming facts (2.30.0), Erase its target
-- table (7.2.0), and each family
-- ignores the keys it does not own exactly as the envelope's
-- unknown-field rule already demands (§1). Promoted here when the NINTH family (erase/1)
-- minted the record + FromJSON pair verbatim for the third time —
-- the scan/trend twins were a banked ledger class until then.
data RowsReq = RowsReq
  { rowsId :: Value
  , rowsOf :: [[Integer]]
  , knobsOf :: [[Integer]]
  , gradesOf :: [[Integer]]
  , namingOf :: Maybe [[Integer]]
  , -- scan/1's call table (6.5.0): the arcs the recursion increment
    -- judges, as indices into `rows`. Nothing = a client that sends
    -- none: no arc, nothing to bump.
    callsOf :: Maybe [[Integer]]
  , rulepackOf :: Rulepack
  , -- scan/1's fence channel (6.4.0, O33): the `knobsFence` value as
    -- it rode — Nothing = the key never came (refused by name at
    -- 9.0.0, `knobsFence: required at 9.0.0`), Just Null = no committed baseline
    -- (unfenced), Just [current, recorded] = the two digests to
    -- compare. Three states, so absent and null are never one.
    fenceOf :: Maybe Value
  , -- whether the request carried `judgedMask` (7.2.0, retired at
    -- 8.0.0, plan v2.32 step 6): scan/1 refuses it by name, the
    -- judged set being the core's language table (CE.Wire.Mask).
    maskSent :: Bool
  , -- erase/1's target table (7.2.0, plan v2.30 step 7b): one
    -- [pathId, start, end] per fact row, the key the target closure
    -- groups rows by. Nothing = the key never came, which erase/1
    -- refuses by name at 9.0.0 (`targets: required at 9.0.0`).
    targetsOf :: Maybe [[Integer]]
  , -- scan/1's events table (7.2.0, plan v2.30 step 7b ③): each
    -- unit's structural events, keyed by its cognitive row, the
    -- three complexity rows derived from them. Nothing = the key
    -- never came, which scan/1 refuses by name at 9.0.0.
    eventsOf :: Maybe [[Integer]]
  }

-- | scan/1's rulepack channel (3.2.0), read off the SAME object: each
-- row's path class, aligned to rows (absent = every row on the
-- global table), and the per-class grade lines
-- [classId, code, warn, fail]. Its own record so the channel's two
-- keys travel as one fact — a family that ignores them ignores one.
data Rulepack = Rulepack
  { rowClassesOf :: Maybe [Integer]
  , overridesOf :: [[Integer]]
  }

instance FromJSON Rulepack where
  parseJSON = withObject "Rulepack" $ \o ->
    Rulepack <$> o .:? "rowClasses" <*> o .:? "gradeOverrides" .!= []

instance FromJSON RowsReq where
  parseJSON = withObject "RowsReq" $ \o ->
    RowsReq
      <$> o .: "id"
      <*> o .: "rows"
      <*> o .:? "knobs" .!= []
      <*> o .:? "grades" .!= []
      <*> o .:? "naming"
      <*> o .:? "callEdges"
      <*> parseJSON (Object o)
      <*> pure (KM.lookup "knobsFence" o)
      <*> pure (judgedMaskSent o)
      <*> o .:? "targets"
      <*> o .:? "events"

-- | One family's bindings for the shared cascade.
data Family req = Family
  { famName :: FamilyName
  -- ^ decode-error prefix ("clone: ...")
  , famId :: req -> Value
  , famOverCap :: req -> Bool
  , famOffence :: req -> Maybe String
  , famDegraded :: req -> B8.ByteString
  , famJudged :: req -> B8.ByteString
  }

-- | The whole cascade for a family with a request type of its own —
-- cap, offence, degraded and judged stay per-family ARGUMENTS (one
-- authority per family), while the Family-literal plumbing lives
-- once: after RowsReq landed, that literal was the last per-family
-- clone the ratchet still charged the table families for, and the
-- thirteenth family (query/1) would have reminted similar/1's copy
-- of it — the write gate named the block, so it moved here.
family ::
  (FromJSON req) =>
  FamilyName ->
  (req -> Value) ->
  (req -> Bool) ->
  (req -> Maybe String) ->
  (req -> B8.ByteString) ->
  (req -> B8.ByteString) ->
  B8.ByteString ->
  Answered
family name ident overCap offence deg jud = respondWith (Family name ident overCap offence deg jud)

-- | A family whose degraded reply is its judged reply with nothing
-- judged (plan v2.33 step 11a): one answer, read twice, gives the
-- family's fields and counts, and the skeleton spells the envelope,
-- the degraded flag and the reason (`tooLarge`) once.
tabled ::
  (FromJSON req) =>
  FamilyName ->
  (req -> Value) ->
  (req -> Bool) ->
  (req -> Maybe String) ->
  (Bool -> req -> ([Pair], [Pair])) ->
  Respond
tabled name ident overCap offence answer proto = family name ident overCap offence (line True) (line False)
 where
  line degraded req = uncurry (resultLine proto name (ident req)) (answer degraded req) (tooLarge name degraded)

-- | A RowsReq family: the shared request type's own id.
rowsFamily ::
  FamilyName ->
  (RowsReq -> Bool) ->
  (RowsReq -> Maybe String) ->
  (RowsReq -> B8.ByteString) ->
  (RowsReq -> B8.ByteString) ->
  B8.ByteString ->
  Answered
rowsFamily name = family name rowsId

-- | The cap a KNOBBED table family counts against: rows and knob
-- rows together (review C15's stance — every request dimension
-- counts). Promoted when the eleventh family (tombstone/1) minted
-- trend/2's lambda verbatim — this module's own promotion rule.
tableCap :: Integer -> RowsReq -> Bool
tableCap cap req = toInteger (length (rowsOf req) + length (knobsOf req)) > cap

-- | A KNOBBED table family (trend/2, tombstone/1): rows and knob rows
-- count against one cap together, the first malformed row in request
-- order is the offence, else the knob table's (CE.Wire.Rows.knobTable,
-- labelled "knob") — rows are facts in request order and none is
-- asked to ascend. trend/2 minted this cascade and tombstone/1 copied
-- it verbatim, so it lives here (this module's own promotion rule).
knobbedRows ::
  FamilyName ->
  Integer ->
  (Int -> [Integer] -> Maybe String) ->
  ([[Integer]] -> Maybe String) ->
  (String -> RowsReq -> B8.ByteString) ->
  (String -> RowsReq -> B8.ByteString) ->
  Respond
knobbedRows name cap rowShape knobsOffence deg jud proto =
  rowsFamily name (tableCap cap) offence (deg proto) (jud proto)
 where
  offence req =
    asum
      [ asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
      , knobsOffence (knobsOf req)
      ]

-- | A KNOBLESS table family (erase/1, audit/1): rows
-- shape-checked per index against a cap, and ANY knob row
-- refused by name — the tunability refusal is a contract
-- statement, not an oversight (a knob that loosened a safety or
-- tolerance predicate would be a licence). Promoted when the
-- tenth family (audit/1) reminted erase/1's violation shell
-- verbatim — this module's own promotion rule.
knoblessRows ::
  FamilyName ->
  Integer ->
  (Int -> [Integer] -> Maybe String) ->
  -- | the family's own contract past the rows (erase/1's target
  -- table, 7.2.0); a family with none passes `const Nothing`
  (RowsReq -> Maybe String) ->
  (RowsReq -> B8.ByteString) ->
  (RowsReq -> B8.ByteString) ->
  B8.ByteString ->
  Answered
knoblessRows name cap rowShape more =
  rowsFamily
    name
    (\req -> toInteger (length (rowsOf req)) > cap)
    violation
 where
  violation req =
    asum
      [ asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
      , asum (zipWith noKnob [0 :: Int ..] (knobsOf req))
      , more req
      ]
  noKnob i _ =
    Just (label "knob" i <> nameWord name <> "/" <> show (nameVersion name) <> " declares no knob codes")

-- | decode → cap check (a complete degraded reply, never a
-- truncated one) → boundary contract (error/contract naming the
-- first offender) → judge.
respondWith ::
  (FromJSON req) =>
  Family req ->
  B8.ByteString ->
  Answered
respondWith fam line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", nameWord (famName fam) <> ": " <> e)
  Right req
    | famOverCap fam req -> Right (famDegraded fam req)
    | Just why <- famOffence fam req -> Left (Just (famId fam req), "contract", why)
    | otherwise -> Right (famJudged fam req)

-- | Fold [code, value] rows through a setter table; rows whose code
-- the table does not own fall through untouched (validation already
-- bounded every code). Promoted from CE.Verdict when the seventh
-- family recloned it verbatim (the twelfth ratchet bite).
applyRows :: [(Integer, Integer -> a -> a)] -> [[Integer]] -> a -> a
applyRows setters = flip (foldl' step)
 where
  step k row = case row of
    [code, v] | Just set <- lookup code setters -> set v k
    _ -> k

-- | Last [code, value] match or the default (later rows win — the
-- applyRows fold order, though validation's ascending rule already
-- forbids duplicate codes within one table).
pick :: [[Integer]] -> Integer -> Integer -> Integer
pick rows code dflt = last (dflt : [v | [c, v] <- rows, c == code])
