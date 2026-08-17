-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The wire-battery harness the respond-driven batteries share
-- (ADR-008 P3 ratchet repayment: ScanProps recloned
-- VerdictWireProps' scaffold line for line — the tenth bite's test
-- half). One check runner, one respond-to-Object decoder, one
-- request editor, one field reader; each battery keeps only its own
-- probes.
module WireHarness (field, replyObjWith, runChecks, setKey) where

import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | The checks-as-a-table runner (the VerdictWireProps shape).
runChecks :: [(String, Bool)] -> IO Bool
runChecks checks = fmap and (mapM one checks)
 where
  one (name, ok) = do
    putStrLn ((if ok then "ok   " else "FAIL ") <> name)
    pure ok

-- | Drive a family's REAL respond and decode the reply object.
replyObjWith ::
  (String -> B8.ByteString -> Either e B8.ByteString) ->
  Value ->
  Maybe Object
replyObjWith respond r = do
  bytes <- either (const Nothing) Just (respond "0.0.1" (BL.toStrict (encode r)))
  Object o <- decodeStrict bytes
  pure o

setKey :: String -> Value -> Value -> Value
setKey k v (Object o) = Object (KM.insert (Key.fromString k) v o)
setKey _ _ v = v

field :: Object -> String -> Maybe Value
field o k = KM.lookup (Key.fromString k) o
