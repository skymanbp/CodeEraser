-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | verdict.request handler (design §2.2): decode, enforce the
-- node/row caps, machine-check the boundary contract (CE.Verdict.Wire)
-- — then judge: join candidates through CE.Verdict.Join, the
-- seven-axis score through CE.Verdict.Score, and the ADR-006 ratchet
-- through CE.Verdict.Ratchet. The baseline crosses the wire VERBATIM
-- from ce-baseline.json (Rust never interprets it, ADR-008); the
-- Wire module is its only reader. The M5-3a stub refused here;
-- M5-3i replaced exactly that refusal with the computation — the
-- graph family's 2a → 2g path, walked a third time.
module CE.Verdict (respond) where

import qualified CE.Dedup.Cost as DedupCost
import CE.Verdict.Cost (softMax, softMin, verdictNodeCap, verdictRowCap)
import CE.Verdict.Candidates (candidates)
import CE.Verdict.Join (bound, severities)
import CE.Verdict.Knobs (effectiveJoin, effectiveKnobs, effectiveRatchet, knobsEcho)
import CE.Verdict.Ratchet (Baseline (..), ratchet, ratchetBound)
import CE.Verdict.Cost (classCocTolCode, classTolCode)
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), classKnobsOf, effectiveWeights, penalties, score, scoreBound)
import CE.Verdict.Soft (softLine)
import CE.Verdict.Baseline (parseBaseline)
import CE.Verdict.Faces (digestKey, failConditions, newBaselineObj, ratchetObj)
import CE.Verdict.Wire (VerdictReq (..), violation)
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M
import Data.Maybe (isJust)

-- | Left = (id to echo, error code, message) for the dispatcher's
-- error encoder; Right = the encoded verdict.result line. The
-- baseline is parsed exactly ONCE here and handed to the cap, the
-- boundary check and the judgment (the M5-close review pair:
-- baseline rows escaped the row cap, and parseBaseline ran twice).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "verdict: " <> e)
  Right req
    | toInteger (length (reqTier req)) > verdictNodeCap
        || toInteger (rowTotal req + baselineRows parsed) > verdictRowCap ->
        Right (tooLarge proto req)
    | Just why <- violation parsed req ->
        Left (Just (reqId req), "contract", why)
    | otherwise ->
        Right (result proto parsed req)
   where
    parsed = parseBaseline (reqBaseline req)

rowTotal :: VerdictReq -> Int
rowTotal req =
  sum
    [ length (reqSim req)
    , length (reqPos req)
    , length (reqChurn req)
    , length (reqCochange req)
    , length (reqCont req)
    , length (reqDisc req)
    , -- the knob tables and the dedup pair count too (review C15:
      -- the declared cap missed these request dimensions, so an
      -- oversized knob table was walked in full by the checker)
      length (reqWeights req)
    , length (reqCeilings req)
    , length (reqThresholds req)
    , length (reqTolerance req)
    , maybe 0 length (reqDedup req)
    , length (reqDedupDistinct req)
    , length (reqJudgedLoc req)
    , length (reqDocFiles req)
    , length (reqClassKnobs req)
    , -- the 6.1.0 export surface was the one table the cap missed
      -- (K47, 6.2.0 — the same C15 debt a second time)
      length (reqSymbols req)
    , -- the 6.4.0 tables: provenance (O40) and the self-loop set (O59)
      maybe 0 length (reqPresent req)
    , maybe 0 length (reqSelfLoops req)
    ]

-- | Baseline rows count toward the SAME row cap as the live tables —
-- a malformed baseline contributes 0 and is refused by violation
-- immediately after the cap admits the request.
baselineRows :: Either String (Maybe Baseline) -> Int
baselineRows (Right (Just (Baseline cont disc _ _))) = length cont + length disc
baselineRows _ = 0

-- | The judged result: join candidates per sim row, the seven axes,
-- the ratchet delta, and the tightened baseline. fail is the
-- ADR-006 conjunction: ratchet (over ceiling past tolerance, or a
-- new discrete member) OR the --fail-under floor — either alone
-- fails (plan: "两者任一 fail 即 fail").
result :: String -> Either String (Maybe Baseline) -> VerdictReq -> B8.ByteString
result proto parsed req =
  BL.toStrict . encode . object $
    [ "proto" .= proto
      , "type" .= ("verdict.result" :: String)
      , "id" .= reqId req
      , "candidates" .= candidates jk req
      , -- the verdict table's severity face (2.33.0, H4): shipped
        -- once, ranked with — the number is the core's
        "joinSeverity" .= [[c, s] | (c, s) <- severities]
      , "score" .= perMille
      , "axes" .= [[c, p] | (c, p) <- pens]
      , "ratchet" .= ratchetObj (isJust (reqPresent req)) r conds
      , "newBaseline" .= newBaselineObj r newSoft (reqKnobsDigest req)
      , -- the EFFECTIVE knob echo (ADR-008): the client asserts the
        -- round trip, and the empty-table default gate pins core
        -- defaults == ce.toml defaults — the drift check the
        -- retired mirrors never had
        "knobs" .= knobsEcho k rk jk dedupFloor (reqJudgedMask req) (cycleRode req)
      , -- batch-7 slice 1 (2.19.0, additive): the core's OWN
        -- admitted-block count from the distinct rows, null when the
        -- rows did not ride (the trend null-absence stance) — the
        -- client proves its filter equal against this, so the
        -- printed report and the gated number can never silently
        -- diverge
        "dedupBlocks"
          .= (if null (reqDedupDistinct req) then Nothing else Just dedupDerived)
      , -- the effective weight table 0..6 (review C3, 2.8.0
        -- additive): the one knob family that had no round trip —
        -- computed by the SAME lookup the score folded with
        "weights" .= effectiveWeights k (reqWeights req)
      , "degraded" .= False
      ]
      -- the class rows echo exactly when they rode (3.1.0): the
      -- client asserts the round trip; a legacy reply keeps its bytes
      <> ["classKnobs" .= reqClassKnobs req | not (null (reqClassKnobs req))]
 where
  k = effectiveKnobs (reqCeilings req) (reqThresholds req)
  rk = effectiveRatchet (reqTolerance req)
  jk = effectiveJoin k (reqThresholds req)
  -- the class rows fold into ONE Map here, once per judgment (3.1.0)
  facts = Facts (reqSim req) (reqPos req) (reqChurn req) (reqCont req) (reqDocFiles req) knobs (maybe [] id (reqSelfLoops req))
  pens = penalties k effSoft facts
  (perMille, _viol) = score k (reqWeights req) pens
  base = either (const Nothing) id parsed
  -- the soft line judging THIS run: the committed one, or (only at
  -- establish) the fresh derivation — which is also what the new
  -- baseline freezes, so the establishing run and every later run
  -- judge with the same S
  effSoft = case base of
    Nothing -> softLine (sSoftK k) softMin softMax (reqJudgedLoc req)
    Just b -> bSoft b
  newSoft = effSoft
  -- the class column is a charging parameter, never a ratchet fact
  -- (plan v2.13 ①): the rows arrive whole so a class may set its own
  -- ALLOWANCE (5.1.0), and the baseline the ratchet writes back is
  -- still three columns — an allowance is not a fact about the tree
  r = ratchet rk classTol (reqPresent req) base (reqCont req) (reqDisc req)
  -- the class allowance by (class, metric) (6.4.0, O37): code 4
  -- answers cognitive complexity where declared, code 3 otherwise —
  -- so a request without code 4 judges exactly as it always did
  classTol c metric =
    (if metric == 1 then M.lookup (c, classCocTolCode) knobs else Nothing)
      <|> M.lookup (c, classTolCode) knobs
  knobs = classKnobsOf (reqClassKnobs req)
  floorFail = maybe False (perMille <) (reqFloor req)
  (dedupFloor, dedupDerived, dedupOver) = dedupLeg req
  -- the fence (5.1.0, plan v2.14 ②): the digest the ceilings were
  -- established under against the one this run declares. Maybe
  -- inequality is the whole rule and it is total — both absent
  -- agrees; a changed rulepack disagrees, and so does declaring one
  -- against a baseline that predates the fence, or removing one the
  -- baseline recorded. Every disagreement wants the same answer: say
  -- so by name, and make a human name the new floor.
  digestDrift = maybe False ((/= reqKnobsDigest req) . bKnobsDigest) base
  conds = failConditions r floorFail dedupOver digestDrift

-- | The second ratchet's leg (ADR-008 P2, split from result at the
-- E01 fn gate when 2.19.0 landed): the pair's shape is already
-- validated, absent = not judged. Since 2.19.0 (batch-7 slice 1)
-- the distinct rows, when they ride, make the core's OWN derivation
-- the judged count — the client's claimed blocks is display, the
-- derivation is the gate. Returns (effective floor, derived count,
-- over-budget).
dedupLeg :: VerdictReq -> (Integer, Integer, Bool)
dedupLeg req = (floor', derived, over)
 where
  floor' = maybe DedupCost.minDistinct id (reqDedupFloor req)
  derived = toInteger (length [d | d <- reqDedupDistinct req, d >= floor'])
  over = case reqDedup req of
    Just [blocks, budget]
      | null (reqDedupDistinct req) -> blocks > budget
      | otherwise -> derived > budget
    _ -> False

-- | Did the cycle floor ride (thresholds code 7)? Its echo and the
-- self-loop table's admission both key on this one fact.
cycleRode :: VerdictReq -> Bool
cycleRode req = any ((== [7]) . take 1) (reqThresholds req)

-- | Over-cap refusal: a well-formed degraded result with the FULL
-- key set, never a truncated judgment.
tooLarge :: String -> VerdictReq -> B8.ByteString
tooLarge proto req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("verdict.result" :: String)
      , "id" .= reqId req
      , "candidates" .= ([] :: [Value])
      , -- constants, not client input — the degraded reply may
        -- still speak them (the C14 defaults posture)
        "joinSeverity" .= [[c, s] | (c, s) <- severities]
      , "score" .= (0 :: Int)
      , "axes" .= ([] :: [Value])
      , "ratchet"
          .= object
            ( [ "added" .= ([] :: [Value])
              , "removed" .= ([] :: [Value])
              , "over" .= ([] :: [Value])
              , "toleranceDrawn" .= ([] :: [Value])
              , -- ADR-008 P1: a gate that could not judge must never
                -- pass, said by the CORE — the degraded reply carries
                -- its own fail semantics; Rust relays, never re-derives
                "fail" .= True
              , "failed" .= (["degraded"] :: [String])
              ]
                -- `present` rode (6.4.0): the key answers here too, so
                -- a reply without it is an older core, never a new one
                -- that judged nothing
                <> ["dropped" .= ([] :: [Value]) | isJust (reqPresent req)]
            )
      , "newBaseline"
          .= object
            ( [ "continuous" .= ([] :: [Value])
              , "discrete" .= ([] :: [Value])
              , "softLine" .= (Nothing :: Maybe Integer)
              ]
                <> digestKey (reqKnobsDigest req)
            )
      , -- defaults: no judgment ran, so no override was applied
        "knobs" .= knobsEcho scoreBound ratchetBound bound DedupCost.minDistinct 0 False
      , "weights" .= effectiveWeights scoreBound []
      , "degraded" .= True
      , "reason" .= ("verdict_too_large" :: String)
      ]
