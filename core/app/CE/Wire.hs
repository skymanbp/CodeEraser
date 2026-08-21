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
module CE.Wire (Family (..), RowsReq (..), applyRows, ascendingOn, knoblessRows, pick, respondWith, rowsFamily, notAscending) where

import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import Data.Foldable (asum)

-- | The [[Integer]]-rows request the table families share: id, the
-- fact rows, and both optional side tables — Trend/Erase read
-- knobs, Scan reads grades, and each family ignores the key it does
-- not own exactly as the envelope's unknown-field rule already
-- demands (§1). Promoted here when the NINTH family (erase/1)
-- minted the record + FromJSON pair verbatim for the third time —
-- the scan/trend twins were a banked ledger class until then.
data RowsReq = RowsReq
  { rowsId :: Value
  , rowsOf :: [[Integer]]
  , knobsOf :: [[Integer]]
  , gradesOf :: [[Integer]]
  }

instance FromJSON RowsReq where
  parseJSON = withObject "RowsReq" $ \o ->
    RowsReq
      <$> o .: "id"
      <*> o .: "rows"
      <*> o .:? "knobs" .!= []
      <*> o .:? "grades" .!= []

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
