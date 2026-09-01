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
module CE.Wire (Family (..), RowsReq (..), Rulepack (..), applyRows, ascendingOn, knoblessRows, pick, respondWith, rowsFamily, notAscending, rowCheck, tableOffence) where

import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import Data.Foldable (asum)

-- | The [[Integer]]-rows request the table families share: id, the
-- fact rows, and the optional side tables — Trend/Erase read knobs,
-- Scan reads grades and naming facts (2.30.0), and each family
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
    -- none, and the reply keeps its legacy bytes.
    callsOf :: Maybe [[Integer]]
  , rulepackOf :: Rulepack
  , -- scan/1's fence channel (6.4.0, O33): the `knobsFence` value as
    -- it rode — Nothing = the key never came (a client that read no
    -- baseline: legacy bytes), Just Null = no committed baseline
    -- (unfenced), Just [current, recorded] = the two digests to
    -- compare. Three states, so absent and null are never one.
    fenceOf :: Maybe Value
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

-- | One family's bindings for the shared cascade.
data Family req = Family
  { famName :: String
  -- ^ decode-error prefix ("clone: ...")
  , famId :: req -> Value
  , famOverCap :: req -> Bool
  , famOffence :: req -> Maybe String
  , famDegraded :: req -> B8.ByteString
  , famJudged :: req -> B8.ByteString
  }

-- | The whole cascade for a RowsReq family — cap, offence, degraded
-- and judged stay per-family ARGUMENTS (one authority per family),
-- while the Family-literal plumbing lives once: after RowsReq
-- landed, that literal was the last per-family clone the ratchet
-- still charged the table families for.
rowsFamily ::
  String ->
  (RowsReq -> Bool) ->
  (RowsReq -> Maybe String) ->
  (RowsReq -> B8.ByteString) ->
  (RowsReq -> B8.ByteString) ->
  B8.ByteString ->
  Either (Maybe Value, String, String) B8.ByteString
rowsFamily name overCap offence deg jud =
  respondWith
    Family
      { famName = name
      , famId = rowsId
      , famOverCap = overCap
      , famOffence = offence
      , famDegraded = deg
      , famJudged = jud
      }

-- | A KNOBLESS table family (erase/1, audit/1): rows
-- shape-checked per index against a cap, and ANY knob row
-- refused by name — the tunability refusal is a contract
-- statement, not an oversight (a knob that loosened a safety or
-- tolerance predicate would be a licence). Promoted when the
-- tenth family (audit/1) reminted erase/1's violation shell
-- verbatim — this module's own promotion rule.
knoblessRows ::
  String ->
  Integer ->
  (Int -> [Integer] -> Maybe String) ->
  (RowsReq -> B8.ByteString) ->
  (RowsReq -> B8.ByteString) ->
  B8.ByteString ->
  Either (Maybe Value, String, String) B8.ByteString
knoblessRows name cap rowShape =
  rowsFamily
    name
    (\req -> toInteger (length (rowsOf req)) > cap)
    violation
 where
  violation req =
    asum
      [ asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
      , asum (zipWith noKnob [0 :: Int ..] (knobsOf req))
      ]
  noKnob i _ =
    Just ("knob " <> show i <> ": " <> name <> "/1 declares no knob codes")

-- | decode → cap check (a complete degraded reply, never a
-- truncated one) → boundary contract (error/contract naming the
-- first offender) → judge.
respondWith ::
  (FromJSON req) =>
  Family req ->
  B8.ByteString ->
  Either (Maybe Value, String, String) B8.ByteString
respondWith fam line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", famName fam <> ": " <> e)
  Right req
    | famOverCap fam req -> Right (famDegraded fam req)
    | Just why <- famOffence fam req -> Left (Just (famId fam req), "contract", why)
    | otherwise -> Right (famJudged fam req)

-- | Strictly-ascending row check with the family's label vocabulary
-- (list Ord is lexicographic) — one comparison, four families.
notAscending :: (Ord a) => String -> Int -> (a, a) -> Maybe String
notAscending what i (prev, cur)
  | prev < cur = Nothing
  | otherwise = Just (what <> " " <> show i <> ": not strictly ascending")

-- | Whole-table ascending pass on a PROJECTION of each row — the
-- identity prefix for docdup pairs (take 2), the code for scan
-- grades (take 1), the whole row for clone pairs and graph edges
-- (id). One zipWith, five call sites: the review-repair batch's own
-- ratchet bite was the projection lambda cloning across families.
-- | One row's whole contract: the right width, then the checks a
-- well-formed row must pass, both wearing the same "<table> <i>: "
-- label. tableOffence below shares the table-level envelope; this is
-- the row-level one under it, and the clone gate named it when
-- graph/1's node row lost its second arm and its three validators
-- collapsed onto one shape.
--
-- The malformed message is the CALLER's whole string, not a template:
-- families say "malformed row (need [..])" and "malformed knob (need
-- [..])", and the wording is each family's contract, not this
-- skeleton's to normalize.
rowCheck ::
  String -> String -> Int -> ([Integer] -> Maybe String) -> Int -> [Integer] -> Maybe String
rowCheck what malformed width checks i row
  | length row /= width = Just (label <> malformed)
  | otherwise = fmap (label <>) (checks row)
 where
  label = what <> " " <> show i <> ": "

-- | One table's whole contract: every row well shaped in request
-- order, then the table strictly ascending. Nine modules held this
-- exact two-step fold — the clone gate named it the moment graph/1
-- added a fifth table — and it earns its place here the same way
-- ascendingOn did one line below (the tenth ratchet bite promoted
-- that one). Order is preserved deliberately: shape errors must
-- surface before ordering errors, so the ascending pass only ever
-- compares rows it can compare.
tableOffence :: (Ord b) => String -> (a -> b) -> (Int -> a -> Maybe String) -> [a] -> Maybe String
tableOffence what proj row rows =
  asum [asum (zipWith row [0 :: Int ..] rows), ascendingOn what proj rows]

ascendingOn :: (Ord b) => String -> (a -> b) -> [a] -> Maybe String
ascendingOn what proj rows =
  asum
    ( zipWith
        (\i (p, c) -> notAscending what i (proj p, proj c))
        [1 :: Int ..]
        (zip rows (drop 1 rows))
    )

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
