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

import CE.Verdict.Cost (verdictNodeCap, verdictRowCap)
import CE.Verdict.Join (Knobs (..), Legs (..), Pos (..), bound, judge)
import CE.Verdict.Ratchet (Baseline (..), RatchetKnobs (..), Ratcheted (..), ratchet, ratchetBound)
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), penalties, score, scoreBound)
import CE.Verdict.Wire (VerdictReq (..), parseBaseline, violation)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M

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
    ]

-- | Baseline rows count toward the SAME row cap as the live tables —
-- a malformed baseline contributes 0 and is refused by violation
-- immediately after the cap admits the request.
baselineRows :: Either String (Maybe Baseline) -> Int
baselineRows (Right (Just (Baseline cont disc))) = length cont + length disc
baselineRows _ = 0

-- | The judged result: join candidates per sim row, the seven axes,
-- the ratchet delta, and the tightened baseline. fail is the
-- ADR-006 conjunction: ratchet (over ceiling past tolerance, or a
-- new discrete member) OR the --fail-under floor — either alone
-- fails (plan: "两者任一 fail 即 fail").
result :: String -> Either String (Maybe Baseline) -> VerdictReq -> B8.ByteString
result proto parsed req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("verdict.result" :: String)
      , "id" .= reqId req
      , "candidates" .= candidates jk req
      , "score" .= perMille
      , "axes" .= [[c, p] | (c, p) <- pens]
      , "ratchet"
          .= object
            [ "added" .= rAdded r
            , "removed" .= rRemoved r
            , "over" .= rOver r
            , "toleranceDrawn" .= rDrawn r
            , "fail" .= failBit
            ]
      , "newBaseline" .= object ["continuous" .= rNewCont r, "discrete" .= rNewDisc r]
      , -- the EFFECTIVE knob echo (ADR-008): the client asserts the
        -- round trip, and the empty-table default gate pins core
        -- defaults == ce.toml defaults — the drift check the
        -- retired mirrors never had
        "knobs" .= knobsEcho k rk jk
      , "degraded" .= False
      ]
 where
  k = effectiveKnobs (reqCeilings req) (reqThresholds req)
  rk = effectiveRatchet (reqTolerance req)
  jk = effectiveJoin k (reqThresholds req)
  pens = penalties k (Facts (reqSim req) (reqPos req) (reqChurn req) (reqCont req))
  (perMille, _viol) = score k (reqWeights req) pens
  base = either (const Nothing) id parsed
  r = ratchet rk base (reqCont req) (reqDisc req)
  floorFail = maybe False (perMille <) (reqFloor req)
  -- the second ratchet's comparison lives HERE (ADR-008 P2): the
  -- pair's shape is already validated, absent = not judged
  dedupOver = case reqDedup req of
    Just [blocks, budget] -> blocks > budget
    _ -> False
  failBit = any snd (failConditions r floorFail dedupOver)

-- | The ADR-006 fail conjunction as NAMED rows (ADR-008 table form):
-- any held condition fails — over ceiling past tolerance, a new
-- discrete member, the --fail-under floor ("either alone fails"),
-- or the dedup blocks-over-budget half (P2: `ce dedup --check`
-- sends the pair; the ce check road never does).
failConditions :: Ratcheted -> Bool -> Bool -> [(String, Bool)]
failConditions r floorFail dedupOver =
  [ ("ratchet_over", not (null (rOver r)))
  , ("discrete_added", not (null (rAdded r)))
  , ("floor", floorFail)
  , ("dedup_budget", dedupOver)
  ]

-- | scoreBound with the request's knob rows applied: the ceilings
-- table drives axes 0/1, the thresholds table its Score-owned codes
-- (3 = cochangeFloor is Join-owned and falls through to
-- effectiveJoin). One (code, setter) row per knob; absent rows keep
-- the Cost.hs DEFAULTS. Two tables, not four: the batch's first
-- ratchet bite was the join/tolerance sibling tables, repaid by
-- effectiveJoin/effectiveRatchet constructing records directly.
effectiveKnobs :: [[Integer]] -> [[Integer]] -> ScoreKnobs
effectiveKnobs ceils thrs =
  applyRows ceilingSetters ceils (applyRows thresholdSetters thrs scoreBound)

ceilingSetters :: [(Integer, Integer -> ScoreKnobs -> ScoreKnobs)]
ceilingSetters = [(0, \v k -> k {sSizeCeil = v}), (1, \v k -> k {sCocCeil = v})]

thresholdSetters :: [(Integer, Integer -> ScoreKnobs -> ScoreKnobs)]
thresholdSetters =
  [ (0, \v k -> k {sDeadIndegCeil = v})
  , (1, \v k -> k {sRewriteNum = v})
  , (2, \v k -> k {sRewriteDen = v})
  , (4, \v k -> k {sViolCost = v})
  , (5, \v k -> k {sDefaultWeight = v})
  , (6, \v k -> k {sScoreScale = v})
  ]

-- | The tolerance legs by position (0 num / 1 den / 2 abs): a
-- record from lookups, not a sibling lambda table.
effectiveRatchet :: [[Integer]] -> RatchetKnobs
effectiveRatchet rows =
  RatchetKnobs (leg 0 rTolNum) (leg 1 rTolDen) (leg 2 rTolAbs)
 where
  leg c f = pick rows c (f ratchetBound)

-- | The join lattice reads the SAME effective rewrite ratio the
-- score judged with (one authority, two readers) plus the cochange
-- floor row (code 3) the score does not own.
effectiveJoin :: ScoreKnobs -> [[Integer]] -> Knobs
effectiveJoin k thrs =
  bound
    { kRewriteNum = sRewriteNum k
    , kRewriteDen = sRewriteDen k
    , kCochangeFloor = pick thrs 3 (kCochangeFloor bound)
    }

-- | Last [code, value] match or the default (later rows win — the
-- applyRows fold order, though validation's ascending rule already
-- forbids duplicate codes within one table).
pick :: [[Integer]] -> Integer -> Integer -> Integer
pick rows code dflt = last (dflt : [v | [c, v] <- rows, c == code])

-- | Fold [code, value] rows through the setter table; rows whose
-- code the table does not own fall through untouched (validation
-- already bounded every code).
applyRows :: [(Integer, Integer -> a -> a)] -> [[Integer]] -> a -> a
applyRows setters = flip (foldl' step)
 where
  step k row = case row of
    [code, v] | Just set <- lookup code setters -> set v k
    _ -> k

-- | Every configurable knob the judgment ran with, echoed so the
-- client can assert the FULL round trip and the empty-table default
-- gate can kill every remaining mirror.
knobsEcho :: ScoreKnobs -> RatchetKnobs -> Knobs -> Value
knobsEcho k rk jk =
  object
    [ "sizeCeil" .= sSizeCeil k
    , "cocCeil" .= sCocCeil k
    , "deadIndegCeil" .= sDeadIndegCeil k
    , "rewriteNum" .= sRewriteNum k
    , "rewriteDen" .= sRewriteDen k
    , "cochangeFloor" .= kCochangeFloor jk
    , "violCost" .= sViolCost k
    , "defaultWeight" .= sDefaultWeight k
    , "scoreScale" .= sScoreScale k
    , "tolNum" .= rTolNum rk
    , "tolDen" .= rTolDen rk
    , "tolAbs" .= rTolAbs rk
    ]

-- | Join-candidate rows, one per sim row (split from result at the
-- E01 line — the leg maps are the candidates' concern alone). The
-- effective Join knobs arrive from the same thresholds table the
-- score reads, so the two judgments share one authority.
candidates :: Knobs -> VerdictReq -> [[Integer]]
candidates jk req =
  [ [u, v, code, bits, mask]
  | row@(u : v : _) <- reqSim req
  , let (code, mask, bits) = judge jk (legsOf row)
  ]
 where
  -- wire flags are structurally 0 at file granularity (entry-ness
  -- rides reachIn; exported-ness is a symbol fact, R6) — the Pos
  -- field stays for the lattice's RG10 guard and its battery
  posMap =
    M.fromList
      [ (u, Pos indeg reachIn 0 sccId)
      | [u, indeg, _outdeg, sccId, _sccSize, reachIn] <- reqPos req
      ]
  churnMap = M.fromList [(u, (ap, rw)) | [u, rw, ap, _, _] <- reqChurn req]
  cochMap = M.fromList [((u, v), c) | [u, v, c] <- reqCochange req]
  legsOf row = case row of
    [u, v, kind, num, den] ->
      Legs
        { lSim = (kind, num, den)
        , lGraphA = M.lookup u posMap
        , lGraphB = M.lookup v posMap
        , lChurnA = M.findWithDefault (0, 0) u churnMap
        , lChurnB = M.findWithDefault (0, 0) v churnMap
        , lCochange = M.lookup (u, v) cochMap
        }
    _ -> error "sim row shape enforced by violation"

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
      , "score" .= (0 :: Int)
      , "axes" .= ([] :: [Value])
      , "ratchet"
          .= object
            [ "added" .= ([] :: [Value])
            , "removed" .= ([] :: [Value])
            , "over" .= ([] :: [Value])
            , "toleranceDrawn" .= ([] :: [Value])
            , -- ADR-008 P1: a gate that could not judge must never
              -- pass, said by the CORE — the degraded reply carries
              -- its own fail semantics; Rust relays, never re-derives
              "fail" .= True
            ]
      , "newBaseline" .= object ["continuous" .= ([] :: [Value]), "discrete" .= ([] :: [Value])]
      , -- defaults: no judgment ran, so no override was applied
        "knobs" .= knobsEcho scoreBound ratchetBound bound
      , "degraded" .= True
      , "reason" .= ("verdict_too_large" :: String)
      ]
