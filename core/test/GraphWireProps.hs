-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | graph/1's REQUEST CONTRACT, as opposed to its judgment: which
-- shapes are refused and by what name, which optional table selects
-- which road, and what the export surface does to a verdict code.
-- Split from GraphProps at the E01 300-line wall when the 4.1.0
-- symbol table arrived — the same seam VerdictProps/VerdictWireProps
-- already sit on, and the same one CE.Graph/CE.Graph.Contract took
-- on the production side.
module GraphWireProps (battery) where

import CE.Graph (respond)
import CE.Graph.Cost (confidence)
import Data.Aeson (Result (..), ToJSON, Value (..), decodeStrict, encode, fromJSON, object, toJSON, (.=))
import Data.Aeson.Key (Key)
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (Pair)
import Data.List (isInfixOf)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import WireHarness (runChecks)

-- | The contract's three faces in the module's own order: what is
-- refused and by what name, which optional table selects which road,
-- what the export surface does to a code.
battery :: IO Bool
battery = runChecks (refusals <> roads <> export)
 where
  refusals =
    [ ("a wrong-width node row is malformed, by row index", malformedNode)
    , ("unres refusals name the offender", unresRefused)
    , ("symbol refusals name the offender", symRefused)
    ]
  roads =
    [ ("the confidence column rides exactly when the ledger does", confRides)
    , ("K5: an empty symbols table is the same BYTES as none", emptyIsAbsent)
    , ("K52: sccFloor rides and echoes; at floor 1 a self-arc singleton is a cycle, a lone node is not", sccFloorRoad)
    ]
  export = [("K9: the export surface moves the CODE and never the dead set", exportRides)]

-- | K52 (6.4.0, O59): three file nodes, node 0 with a self-arc. The
-- shipped floor (absent) lists no cycle and echoes no `sccFloor`; at
-- floor 1 the reply echoes the floor and lists node 0 alone (node 1
-- has no arc; node 2 none either); floor 0 refuses by name. The
-- verdict's cycleFloor reads the SAME number (VerdictWireProps K50).
sccFloorRoad :: Bool
sccFloorRoad =
  cyclesOf (looped []) == Just []
    && fmap (KM.member "sccFloor") (objOf (looped [])) == Just False
    && cyclesOf (looped [floorOf 1]) == Just [[0]]
    && (objOf (looped [floorOf 1]) >>= KM.lookup "sccFloor") == Just (toJSON (1 :: Int))
    && refusedGraph (looped [floorOf 0]) "sccFloor: below 1"
 where
  floorOf n = "sccFloor" .= (n :: Int)
  looped extra =
    BL.toStrict . encode . object $
      [ "id" .= (1 :: Int)
      , "nodes" .= ([[0, 0, 0], [0, 0, 0], [0, 0, 0]] :: [[Integer]])
      , "edges" .= ([[0, 0, 0, 0], [1, 2, 0, 0]] :: [[Integer]])
      , "pos" .= ([] :: [Value])
      ]
        <> extra
  refusedGraph req want = case respond "6.4.0" req of
    Left (_, code, msg) -> code == "contract" && want `isInfixOf` msg
    Right _ -> False

-- | The whole reply object of one graph request.
objOf :: B8.ByteString -> Maybe (KM.KeyMap Value)
objOf req = do
  Right bytes <- pure (respond "6.4.0" req)
  Object o <- decodeStrict bytes
  pure o

-- | The member lists of the reply's cycle table, in reply order.
cyclesOf :: B8.ByteString -> Maybe [[Integer]]
cyclesOf req = do
  o <- objOf req
  v <- KM.lookup "cycles" o
  case fromJSON v :: Result [(Integer, [Integer])] of
    Success rows -> pure (map snd rows)
    _ -> Nothing

-- | The pre-5.0.0 four-column row. It used to make this table "mixed
-- arity"; with one legal arity it is simply the second row that is
-- wrong, and the refusal says which one.
malformedNode :: Bool
malformedNode = case respond "6.4.0" req of
  Left (_, code, msg) ->
    code == "contract" && msg == "node 1: malformed row (need [lang,kind,roles])"
  Right _ -> False
 where
  req = fixtureReq [[0, 0, 0], [0, 0, 0, 0]] []

-- | Three entry-less file nodes, all dead (unref_private): lang 0
-- fully resolved -> vouched 2; lang 4 with unresolved sites ->
-- unvouched 0; lang 6 absent from the ledger -> vacuous 1. The SAME
-- request without the ledger answers two-column rows — the legacy
-- road is byte-shaped, not merely value-equal. The kernel rule is
-- pinned beside the wire: (0,0) is vacuous, not vouched.
confRides :: Bool
confRides =
  deadOf (graphReq (Just [[0, 0, 3], [4, 2, 9]]))
    == Just (toJSON ([[0, 1, 2], [1, 1, 0], [2, 1, 1]] :: [[Integer]]))
    && deadOf (graphReq Nothing) == Just (toJSON ([[0, 1], [1, 1], [2, 1]] :: [[Integer]]))
    && map (confidence [[0, 0, 3], [4, 2, 9]]) [0, 4, 6] == [2, 0, 1]
    && confidence [[5, 0, 0]] 5 == 1

-- | The `dead` table of one reply — shared by every prop that reads
-- a verdict out of the wire rather than out of Dead.verdicts.
deadOf :: B8.ByteString -> Maybe Value
deadOf req = do
  Right bytes <- pure (respond "6.4.0" req)
  Object o <- decodeStrict bytes
  KM.lookup "dead" o

unresRefused :: Bool
unresRefused =
  and
    [ refusedMsg (Just [[7, 0, 0]]) "unres 0: lang outside the judged set"
    , refusedMsg (Just [[0, 4, 3]]) "unres 0: unresolved above total"
    , refusedMsg (Just [[0, 1]]) "unres 0: malformed row (need [lang,unresolved,total])"
    , refusedMsg (Just [[4, 0, 1], [0, 0, 1]]) "unres 1: not strictly ascending"
    ]
 where
  refusedMsg unres want = case respond "6.4.0" (graphReq unres) of
    Left (_, code, msg) -> code == "contract" && msg == want
    Right _ -> False

-- | Two entry-less file nodes, neither referenced: dead either way.
-- The symbols table only moves node 0's CODE from 1 (unref_private)
-- to 2 (unref_public) — a pair that had NO producer before this
-- minor, because bit 0 is never set at file granularity
-- (cli/src/graph/deadcode/flags.rs:9 `symbol fact`). A visibility word
-- without the
-- export bit names no surface, so it moves nothing.
exportRides :: Bool
exportRides =
  and
    [ deadOf (symReq Nothing) == plain
    , deadOf (symReq (Just [])) == plain
    , deadOf (symReq (Just [[0, 1]])) == Just (toJSON ([[0, 2], [1, 1]] :: [[Integer]]))
    , deadOf (symReq (Just [[0, 2]])) == plain
    , -- and the DEAD SET never moved: both nodes, every time
      deadOf (symReq (Just [[0, 1], [1, 1]])) == Just (toJSON ([[0, 2], [1, 2]] :: [[Integer]]))
    ]
 where
  plain = Just (toJSON ([[0, 1], [1, 1]] :: [[Integer]]))

-- | K5: absence and emptiness are one road, not two — the whole
-- reply, byte for byte, is what a pre-4.1.0 client already got.
emptyIsAbsent :: Bool
emptyIsAbsent = respond "6.4.0" (symReq (Just [])) == respond "6.4.0" (symReq Nothing)

symRefused :: Bool
symRefused =
  and
    [ refusedSym [[2, 1]] "symbol 0: node out of range"
    , refusedSym [[0, -1]] "symbol 0: negative field"
    , refusedSym [[0]] "symbol 0: malformed row (need [node,visibility])"
    , refusedSym [[1, 0], [0, 0]] "symbol 1: not strictly ascending"
    , refusedSym [[0, 1], [0, 1]] "symbol 1: not strictly ascending"
    ]
 where
  refusedSym syms want = case respond "6.4.0" (symReq (Just syms)) of
    Left (_, code, msg) -> code == "contract" && msg == want
    Right _ -> False

-- | The export-surface fixture request: two entry-less file nodes,
-- no edges; the symbols table rides when given.
symReq :: Maybe [[Integer]] -> B8.ByteString
symReq syms = fixtureReq [[0, 0, 0], [0, 0, 0]] (optional "symbols" syms)

-- | An edgeless request over the given node table, plus whichever
-- side table the prop is about. THREE copies of this builder were a
-- T2 clone chain by this repo's own gate on the first draft — what
-- differs between the props is the nodes and the side table, never
-- the procedure.
fixtureReq :: [[Integer]] -> [Pair] -> B8.ByteString
fixtureReq nodes extra =
  BL.toStrict . encode . object $
    [ "id" .= (1 :: Int)
    , "nodes" .= nodes
    , "edges" .= ([] :: [Value])
    , "pos" .= ([] :: [Value])
    ]
      <> extra

-- | A side table under `key`, or no key at all when it did not ride.
optional :: (ToJSON a) => Key -> Maybe a -> [Pair]
optional key = maybe [] pure . fmap (key .=)

-- | The confRides fixture request: three entry-less file nodes of
-- langs 0/4/6, no edges; the ledger rides when given.
graphReq :: Maybe [[Integer]] -> B8.ByteString
graphReq unres = fixtureReq [[0, 0, 0], [4, 0, 0], [6, 0, 0]] (optional "unres" unres)

-- | 200 seeded graphs: (n, arcs with rungs, flags per node).
