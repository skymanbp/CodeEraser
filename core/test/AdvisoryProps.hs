-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | graph/1's ADVISORY tables (6.2.0, plan v2.17 L round piece (6)):
-- the `unmentioned` / `mounts` request contract, the two caps'
-- interaction with refusal and degradation, the folds of
-- CE.Graph.Advisory under ablation, and the iron rule that none of
-- it can move a verdict. Its own module beside GraphWireProps for the
-- same reason CE.Graph.Advisory sits beside CE.Graph.Contract: the
-- advisory is a class of its own, and Spec.hs is at its size line.
-- Legs: K16 (core half), K19, K33, K35, K36.
module AdvisoryProps (battery) where

import CE.Graph (respond)
import qualified CE.Graph.Advisory as Advisory
import CE.Graph.Cost (exemptCategories, mountCap, nodeCap, unmentionedCap, unmentionedHardCap, unmentionedVisMask)
import Data.Aeson (Value (..), decodeStrict, encode, object, toJSON, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import WireHarness (field, fieldsOf, refusedBy, replyObjWith, runChecks)

battery :: IO Bool
battery = runChecks [(leg <> ": " <> name, ok) | (leg, name, ok) <- legs]

-- | Every probe under the sealed criterion's K leg it discharges —
-- the leg is a column here, not a prefix baked into the name, so the
-- criterion's row can be grepped from the battery.
legs :: [(String, String, Bool)]
legs =
  [ ("K33", "the seven mounts refusals name the offender", mountRefused)
  , ("K35", "the four unmentioned refusals name the offender", unmentionedRefused)
  , ("K33/K35", "the pairing refusal stands before every row of every table", pairingFirst)
  , ("K16", "the advisory keys ride exactly when the table rode", keysRide)
  , ("K33", "the advisory never moves the dead set, under any fold", ironRule)
  , ("K33", "the code order 1 > 2 > 3 > 0 and its folds", codeOrder)
  , ("K33", "a node without a mounts row reads [0,0,0]", zeroRow)
  , ("K19/K36", "the mask and the exempt list are the only filters", knobs)
  , ("K35", "the four degradation cells and the drop", capCells)
  , ("K33", "the caps price their own tables, never the nodes", ownCaps)
  ]

-- | A graph request over `nodes` (entry-less file nodes unless said
-- otherwise), no edges, plus the side tables the leg is about.
req :: [[Integer]] -> [Pair] -> Value
req nodes extra =
  object
    ( [ "proto" .= ("6.3.0" :: String)
      , "type" .= ("graph.request" :: String)
      , "id" .= (1 :: Int)
      , "nodes" .= nodes
      , "edges" .= ([] :: [Value])
      , "pos" .= ([] :: [Value])
      ]
        <> extra
    )

-- | Both advisory tables on `n` entry-less file nodes.
both :: Int -> [[Integer]] -> [[Integer]] -> Value
both n mounts unmentioned =
  req (replicate n [0, 0, 0]) ["mounts" .= mounts, "unmentioned" .= unmentioned]

mountRefused :: Bool
mountRefused =
  and
    [ refusedBy respond (both 2 [[0, 0, 0]] []) "mount 0: malformed row (need [node,private,total,bits])"
    , refusedBy respond (both 2 [[0, -1, 0, 0]] []) "mount 0: negative field"
    , refusedBy respond (both 2 [[0, 2, 1, 0]] []) "mount 0: private above total"
    , refusedBy respond (both 2 [[2, 0, 0, 0]] []) "mount 0: node out of range"
    , refusedBy respond (both 2 [[1, 0, 0, 0], [0, 0, 0, 0]] []) "mount 1: not strictly ascending"
    , -- one node, two rows: under a whole-row projection the second
      -- would pass as merely later (§4, F6-5)
      refusedBy respond (both 2 [[0, 1, 2, 0], [0, 2, 3, 0]] []) "mount 1: not strictly ascending"
    , refusedBy respond (req [[0, 0, 0]] ["mounts" .= [[0, 0, 0, 0 :: Integer]]]) "mounts: unmentioned table required alongside"
    , refusedBy respond (req [[0, 0, 0]] ["unmentioned" .= ([] :: [Value])]) "unmentioned: mounts table required alongside"
    , -- total = 0 is legal: zero mounts, the row every non-Rust file sends
      judgedWith (both 1 [[0, 0, 0, 0]] [])
    ]

unmentionedRefused :: Bool
unmentionedRefused =
  and
    [ refusedBy respond (both 2 m [[0, 3]]) "unmentioned 0: malformed row (need [node,vis,conv])"
    , refusedBy respond (both 2 m [[0, 3, -1]]) "unmentioned 0: negative field"
    , refusedBy respond (both 2 m [[2, 3, 0]]) "unmentioned 0: node out of range"
    , refusedBy respond (both 2 m [[1, 3, 0], [0, 3, 0]]) "unmentioned 1: not strictly ascending"
    , refusedBy respond (both 2 m [[0, 3, 0], [0, 3, 0]]) "unmentioned 1: not strictly ascending"
    ]
 where
  m = [[0, 0, 0, 0], [1, 0, 0, 0]]

-- | The pairing message wins over a malformed node row, a malformed
-- row of the lone table itself, and — because famOverCap precedes
-- famOffence — loses only to the hard cap.
pairingFirst :: Bool
pairingFirst =
  and
    [ refusedBy respond (req [[0, 0, 0], [0, 0, 0, 0]] ["unmentioned" .= [[0, 3, 0 :: Integer]]]) "unmentioned: mounts table required alongside"
    , refusedBy respond (req [[0, 0, 0]] ["unmentioned" .= [[0, 3 :: Integer]]]) "unmentioned: mounts table required alongside"
    , refusedBy respond (req [[0, 0, 0]] ["mounts" .= [[0 :: Integer]]]) "mounts: unmentioned table required alongside"
    , reason (lone (fromInteger unmentionedHardCap + 1) "[0,0,0]" "unmentioned") == Just "graph_too_large"
    ]

-- | K16, the core half: no `unmentioned` table => neither key, on the
-- judged and the degraded road alike; an EMPTY table with its mounts
-- partner => `exportUnmentioned` present and `[]`, no drop flag.
keysRide :: Bool
keysRide =
  and
    [ fieldsOf respond (req two []) keys == Just [Nothing, Nothing]
    , fieldsOf respond (req two ["mounts" .= m, "unmentioned" .= ([] :: [Value])]) keys
        == Just [Just (toJSON ([] :: [Value])), Nothing]
    , fieldsOf respond (req (replicate (fromInteger nodeCap + 1) [0, 0, 0]) []) ("degraded" : keys)
        == Just [Just (Bool True), Nothing, Nothing]
    , -- the degraded road with BOTH tables aboard: still neither key
      fieldsOf respond (req (replicate (fromInteger nodeCap + 1) [0, 0, 0]) ["mounts" .= m, "unmentioned" .= ([] :: [Value])]) ("degraded" : keys)
        == Just [Just (Bool True), Nothing, Nothing]
    ]
 where
  two = [[0, 0, 0], [0, 0, 0]]
  m = [[0, 0, 0, 0], [1, 0, 0, 0 :: Integer]]
  keys = ["exportUnmentioned", "unmentionedDropped"]

-- | The §0 iron rule as a counterfactual: two entry-less nodes are
-- dead with no tables, with both tables, with tables that between
-- them yield every code (0 and 2; 1 twice — both folds; 3 and 0),
-- with a dropped table — the same two rows every time.
ironRule :: Bool
ironRule =
  all
    (\extra -> deadOf (req two extra) == deadOf (req two []))
    [ ["mounts" .= m, "unmentioned" .= ([] :: [Value])]
    , ["mounts" .= m, "unmentioned" .= [[0, 3, 0], [1, 7, 0 :: Integer]]]
    , ["mounts" .= [[0, 1, 1, 0], [1, 0, 0, 2 :: Integer]], "unmentioned" .= open]
    , ["mounts" .= reexported, "unmentioned" .= open]
    , ["mounts" .= [[0, 0, 0, 0 :: Integer]], "unmentioned" .= dropRows]
    ]
    && deadOf (req two []) == Just (toJSON ([[0, 1], [1, 1]] :: [[Integer]]))
 where
  two = [[0, 0, 0], [0, 0, 0]]
  m = [[0, 0, 0, 0], [1, 0, 0, 0 :: Integer]]
  -- node 0 a re-export target with no private mount, node 1 bare
  reexported = [[0, 0, 0, 1], [1, 0, 0, 0 :: Integer]]
  open = [[0, 3, 0], [1, 3, 0 :: Integer]]
  dropRows = [[0, 3, c] | c <- [0 .. unmentionedCap]]

-- | Every cell of the code order through the real reply, plus the
-- collision cells (a private mount that is also pkg-private; a
-- restricted declaration in a re-exported file; a re-exported file
-- whose mounts are all private — the re-export bit clears
-- mountedPrivate and the row lands on 3, not 1; a restricted
-- declaration in a privately mounted file — 1 beats 2, the
-- main_cli.rs shape) — and the folds by name, so an ablation of one
-- fold or a swap of two guards is visible as one moved cell.
codeOrder :: Bool
codeOrder =
  advisoryOf (req (replicate 9 [0, 0, 0]) ["mounts" .= mounts, "unmentioned" .= rows])
    == Just (toJSON ([[0, 3, 0, 0], [1, 3, 0, 1], [2, 3, 0, 1], [3, 7, 0, 2], [4, 3, 0, 3], [5, 3, 0, 3], [6, 3, 0, 1], [7, 7, 0, 2], [8, 7, 0, 1]] :: [[Integer]]))
    && map (uncurry Advisory.code) cells == [0, 1, 1, 2, 3, 3, 1, 2, 1]
    && Advisory.mountedPrivate [1, 1, 0]
    && not (Advisory.mountedPrivate [1, 2, 0])
    && not (Advisory.mountedPrivate [0, 0, 0])
    && not (Advisory.mountedPrivate [1, 1, 1])
    && Advisory.pkgPrivate [0, 0, 2]
    && Advisory.reexported [0, 0, 1]
 where
  -- node: 0 lib root (zero mounts) / 1 private mount / 2 pkg-private /
  -- 3 restricted / 4 re-exported / 5 re-exported AND privately
  -- mounted / 6 private mount AND pkg-private / 7 restricted in a
  -- re-exported file / 8 restricted in a privately mounted file
  mounts =
    [ [0, 0, 0, 0], [1, 1, 1, 0], [2, 0, 0, 2], [3, 0, 0, 0], [4, 0, 0, 1], [5, 1, 1, 1], [6, 1, 1, 2], [7, 0, 0, 1], [8, 1, 1, 0 :: Integer]
    ]
  rows = [[n, if n `elem` [3, 7, 8] then 7 else 3, 0] | n <- [0 .. 8 :: Integer]]
  cells = zip [[p, t, b] | [_, p, t, b] <- mounts] (map (!! 1) rows)

-- | K33's zero-row cell: five nodes, one mounts row, an unmentioned
-- row for a node the table never names — every fold reads false and
-- the row lands on code 0; the seven refusals are untouched by it.
zeroRow :: Bool
zeroRow =
  advisoryOf (both 5 [[0, 0, 0, 0]] [[3, 3, 0]]) == Just (toJSON ([[3, 3, 0, 0]] :: [[Integer]]))
    && Advisory.judge unmentionedVisMask exemptCategories [] [[3, 3, 0]] == [[3, 3, 0, 0]]

-- | K19: a category hit silences the row; dropping that category from
-- the exempt list brings exactly that row back; bit 11 never exempts;
-- the mask 1<<2 keeps only restricted rows. K36: exemption reads the
-- category word alone — a row with conv 0 and every visibility bit
-- set is judged — and the list never reaches bit 11.
knobs :: Bool
knobs =
  and
    [ judge exemptCategories rows == [[0, 3, 0, 0], [3, 3, 2048, 0]]
    , judge (filter (/= 1) exemptCategories) rows == [[0, 3, 0, 0], [1, 3, 2, 0], [3, 3, 2048, 0]]
    , judge [] rows == [[0, 3, 0, 0], [1, 3, 2, 0], [2, 3, 1024, 0], [3, 3, 2048, 0]]
    , Advisory.judge 4 exemptCategories [] [[0, 3, 0], [1, 7, 0], [2, 4, 0]] == [[1, 7, 0, 2], [2, 4, 0, 2]]
    , Advisory.judge unmentionedVisMask exemptCategories [] [[0, 1, 0], [1, 2, 0]] == []
    , -- every visibility bit set, empty category word: judged (and
      -- coded 2 by its restricted bit) — visibility never exempts
      Advisory.judge unmentionedVisMask exemptCategories [] [[0, 4095, 0]] == [[0, 4095, 0, 2]]
    , all (`elem` [0 .. 10]) exemptCategories && 11 `notElem` exemptCategories
    , unmentionedVisMask == 3
    ]
 where
  judge exempt = Advisory.judge unmentionedVisMask exempt []
  rows = [[0, 3, 0], [1, 3, 2], [2, 3, 1024], [3, 3, 2048]]

-- | K35's four cells: past the hard cap the request degrades by name
-- (malformed rows and a missing partner included — famOverCap runs
-- first); past the soft cap a malformed row still refuses first;
-- past the soft cap with legal rows the table is DROPPED and said
-- so while `degraded` and `fail` stand exactly where the legacy
-- reply puts them (the iron rule on the drop road).
capCells :: Bool
capCells =
  and
    [ reason (lone hard "[0,0,0]" "unmentioned") == Just "graph_too_large"
    , reason (lone hard "[0,0]" "unmentioned") == Just "graph_too_large"
    , refusedBy respond (both 1 [[0, 0, 0, 0]] (dropRows <> [[0, 3]])) "unmentioned 131073: malformed row (need [node,vis,conv])"
    , fieldsOf respond dropped ["exportUnmentioned", "unmentionedDropped", "degraded", "fail"]
        == Just [Just (toJSON ([] :: [Value])), Just (Bool True), Just (Bool False), Just (Bool True)]
    ]
 where
  hard = fromInteger unmentionedHardCap + 1
  dropRows = [[0, 3, c] | c <- [0 .. unmentionedCap]]
  dropped = both 1 [[0, 0, 0, 0]] dropRows

-- | K33's cap legs: a full mounts table beside an empty unmentioned
-- one is judged; one row over the mount cap degrades by name (a
-- duplicate node row — legal rows cannot exceed the cap, and the cap
-- runs before validation); and at the node boundary the two advisory
-- tables change nothing — the same verdict with and without them,
-- and one more node degrades both requests alike.
ownCaps :: Bool
ownCaps =
  and
    [ judgedWith (both n fullMounts [])
    , reason (lone (fromInteger mountCap + 1) "[0,0,0,0]" "mounts") == Just "graph_too_large"
    , deadOf (boundary 0 True) == deadOf (boundary 0 False) && deadOf (boundary 0 False) /= Nothing
    , reasonOf (boundary 1 True) == Just "graph_too_large" && reasonOf (boundary 1 False) == Just "graph_too_large"
    ]
 where
  n = fromInteger mountCap
  fullMounts = [[i, 0, 0, 0] | i <- [0 .. mountCap - 1]]
  -- the first cap disjunct is nodes + unres (Graph.hs) and the
  -- production road always sends `unres`, so the boundary is
  -- nodeCap - |unres| with a one-row ledger (W5-F15)
  boundary over tables =
    req
      (replicate (fromInteger nodeCap - 1 + over) [0, 0, 0])
      ( ("unres" .= [[0, 0, 1 :: Integer]])
          : (if tables then ["mounts" .= [[0, 0, 0, 0 :: Integer]], "unmentioned" .= ([] :: [Value])] else [])
      )

-- | A one-node request whose `key` table repeats `row` `count` times
-- as raw bytes — the Spec.hs over-cap probe shape (a 524k-row Value
-- has no business being built for a cap check).
lone :: Int -> B8.ByteString -> B8.ByteString -> B8.ByteString
lone count row key =
  "{\"proto\":\"6.3.0\",\"type\":\"graph.request\",\"id\":1,\"nodes\":[[0,0,0]],\"edges\":[],\"pos\":[],\""
    <> key
    <> "\":["
    <> B8.intercalate "," (replicate count row)
    <> "]}"

-- | The degraded reason of a raw-bytes request (the over-cap probes
-- above are built as bytes, so this reads them as bytes).
reason :: B8.ByteString -> Maybe Value
reason raw = do
  Right out <- pure (respond "6.3.0" raw)
  Object o <- decodeStrict out
  field o "reason"

reasonOf :: Value -> Maybe Value
reasonOf = reason . BL.toStrict . encode

deadOf :: Value -> Maybe Value
deadOf r = replyObjWith respond r >>= (`field` "dead")

advisoryOf :: Value -> Maybe Value
advisoryOf r = replyObjWith respond r >>= (`field` "exportUnmentioned")

judgedWith :: Value -> Bool
judgedWith r = (replyObjWith respond r >>= (`field` "degraded")) == Just (Bool False)
