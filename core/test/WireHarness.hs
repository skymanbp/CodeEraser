-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The wire-battery harness the respond-driven batteries share
-- (ADR-008 P3 ratchet repayment: ScanProps recloned
-- VerdictWireProps' scaffold line for line — the tenth bite's test
-- half). One check runner, one respond-to-Object decoder, one
-- request editor, one field reader; each battery keeps only its own
-- probes.
module WireHarness (battery, degradedFace, field, fieldsOf, refusedBy, replyObjWith, rowsRequest, runChecks, runLegs, setKey, tabledRequest) where

import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.List (isInfixOf)
import qualified Data.Set as Set
import Data.Version (showVersion)
import Paths_ce_core (version)

-- | Same single source as Spec/Main — never an injected literal.
coreVersion :: String
coreVersion = showVersion version

-- | The checks-as-a-table runner (the VerdictWireProps shape).
runChecks :: [(String, Bool)] -> IO Bool
runChecks checks = fmap and (mapM one checks)
 where
  one (name, ok) = do
    putStrLn ((if ok then "ok   " else "FAIL ") <> name)
    pure ok

-- | A reference battery over numbered cases: the two headline checks
-- (every case agrees; the cases reach every required answer shape),
-- then the first five disagreeing cases and each shape never reached.
battery :: (String, String) -> [Maybe String] -> [String] -> Set.Set String -> [(String, Bool)]
battery (agreed, reaches) verdicts required reached =
  [(agreed, null off), (reaches, null missing)] <> map named (take 5 off) <> map unreached missing
 where
  off = [(k, d) | (k, Just d) <- zip [1 :: Int ..] verdicts]
  missing = filter (`Set.notMember` reached) required
  named (k, d) = ("  case " ++ show k ++ ": " ++ d, False)
  unreached m = ("  never reached: " ++ m, False)

-- | The table runner on two PARALLEL lists — names and probes — with
-- the alignment leg first. A (name, probe) row per leg is a T2 clone
-- of every sibling battery past six rows by this repo's own measure
-- (the erase and structure batteries met the gate at 7.2.0), while a
-- list of names and a list of probes each sit below the distinct-
-- token floor; zip's silent truncation is what the first leg guards.
runLegs :: [String] -> [Bool] -> IO Bool
runLegs names probes =
  runChecks (("every leg names one probe", length names == length probes) : zip names probes)

-- | A request envelope carrying NAMED [[Integer]] fact tables — the
-- structure family's wireReq shape. Promoted when the modularity
-- battery reminted the family fixture's `object` head verbatim, the
-- same way the dedup gate named rowsRequest just below.
tabledRequest :: String -> String -> [(String, [[Integer]])] -> Value
tabledRequest protoV kind tables =
  object
    ( ["proto" .= protoV, "type" .= kind, "id" .= (1 :: Int)]
        <> [Key.fromString name .= rows | (name, rows) <- tables]
    )

-- | A [[Integer]]-rows request envelope — the scan/trend batteries'
-- shared wireReq shape (the trend family's landing recloned scan's
-- line for line; fifteenth-bite test-half repayment). One envelope,
-- two faces: the single table this one carries is named `rows`.
rowsRequest :: String -> String -> [[Integer]] -> Value
rowsRequest protoV kind rows = tabledRequest protoV kind [("rows", rows)]

-- | Drive a family's REAL respond and decode the reply object.
replyObjWith ::
  (String -> B8.ByteString -> Either e B8.ByteString) ->
  Value ->
  Maybe Object
replyObjWith respond r = do
  bytes <- either (const Nothing) Just (respond coreVersion (BL.toStrict (encode r)))
  Object o <- decodeStrict bytes
  pure o

-- | Several reply fields in one read — the probe-local "reply, then
-- project a tuple" do-blocks kept recloning this walk (the dedup
-- gate caught its author again at trend/2).
fieldsOf ::
  (String -> B8.ByteString -> Either e B8.ByteString) ->
  Value ->
  [String] ->
  Maybe [Maybe Value]
fieldsOf respond r ks = do
  o <- replyObjWith respond r
  pure (map (field o) ks)

setKey :: String -> Value -> Value -> Value
setKey k v (Object o) = Object (KM.insert (Key.fromString k) v o)
setKey _ _ v = v

field :: Object -> String -> Maybe Value
field o k = KM.lookup (Key.fromString k) o

-- | The degraded-face posture every capped family shares: a
-- refused request answers a COMPLETE reply that FAILS, with the
-- named table EMPTY (a plan the core refused to judge licenses
-- nothing) and its reason. Promoted when the audit battery
-- reminted the erase battery's probe line for line.
degradedFace ::
  (String -> B8.ByteString -> Either e B8.ByteString) ->
  Value ->
  String ->
  String ->
  Bool
degradedFace respond r tableKey reason = case replyObjWith respond r of
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
      && field o tableKey == Just (toJSON ([] :: [Value]))
      && field o "reason" == Just (toJSON reason)
  Nothing -> False

-- | A named contract refusal through a family's REAL respond — the
-- fourth pasted copy of this predicate (structure joining scan and
-- verdict) triggered the promotion (twelfth bite, test half).
refusedBy ::
  (String -> B8.ByteString -> Either (a, String, String) b) ->
  Value ->
  String ->
  Bool
refusedBy respond r want = case respond coreVersion (BL.toStrict (encode r)) of
  Left (_, code, msg) -> code == "contract" && want `isInfixOf` msg
  Right _ -> False
