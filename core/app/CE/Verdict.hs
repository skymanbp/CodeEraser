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
import CE.Verdict.Join (Legs (..), Pos (..), bound, judge)
import CE.Verdict.Ratchet (Ratcheted (..), ratchet, ratchetBound)
import CE.Verdict.Score (Facts (..), penalties, score, scoreBound)
import CE.Verdict.Wire (VerdictReq (..), parseBaseline, violation)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M

-- | Left = (id to echo, error code, message) for the dispatcher's
-- error encoder; Right = the encoded verdict.result line.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "verdict: " <> e)
  Right req
    | toInteger (length (reqTier req)) > verdictNodeCap
        || toInteger (rowTotal req) > verdictRowCap ->
        Right (tooLarge proto req)
    | Just why <- violation req ->
        Left (Just (reqId req), "contract", why)
    | otherwise ->
        Right (result proto req)

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

-- | The judged result: join candidates per sim row, the seven axes,
-- the ratchet delta, and the tightened baseline. fail is the
-- ADR-006 conjunction: ratchet (over ceiling past tolerance, or a
-- new discrete member) OR the --fail-under floor — either alone
-- fails (plan: "两者任一 fail 即 fail").
result :: String -> VerdictReq -> B8.ByteString
result proto req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("verdict.result" :: String)
      , "id" .= reqId req
      , "candidates" .= candidates req
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
      , "degraded" .= False
      ]
 where
  pens = penalties scoreBound (Facts (reqSim req) (reqPos req) (reqChurn req) (reqCont req))
  (perMille, _viol) = score scoreBound (reqWeights req) pens
  base = either (const Nothing) id (parseBaseline (reqBaseline req))
  r = ratchet ratchetBound base (reqCont req) (reqDisc req)
  floorFail = maybe False (perMille <) (reqFloor req)
  failBit = not (null (rOver r)) || not (null (rAdded r)) || floorFail

-- | Join-candidate rows, one per sim row (split from result at the
-- E01 line — the leg maps are the candidates' concern alone).
candidates :: VerdictReq -> [[Integer]]
candidates req =
  [ [u, v, code, bits, mask]
  | row@(u : v : _) <- reqSim req
  , let (code, mask, bits) = judge bound (legsOf row)
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
            , "fail" .= False
            ]
      , "newBaseline" .= object ["continuous" .= ([] :: [Value]), "discrete" .= ([] :: [Value])]
      , "degraded" .= True
      , "reason" .= ("verdict_too_large" :: String)
      ]
