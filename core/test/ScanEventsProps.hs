-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | scan/1's events road (7.2.0, plan v2.30 step 7b ③): the
-- whitepaper register — contracts/fixtures/scan/whitepaper.ndjson,
-- the page's worked examples in every judged language that parses,
-- each line's events the Rust emitter's, which
-- cli/tests/it/sonar_whitepaper.rs pins to the line's source —
-- replayed through CE.Scan.Complexity.fold, so every margin value
-- the page prints is reached here without a parser: the rules' own
-- battery. Then the rules the register cannot single out, the road
-- through the REAL respond (the derived echo, the grade on it, the
-- cycle charge landing on the derived value, the legacy and degraded
-- faces) and every refusal by name. Its own module beside ScanProps
-- and ScanCyclesProps for the reason those two are apart: the
-- derivation is a class of its own, and both stand at their size
-- line.
module ScanEventsProps (battery) where

import CE.Scan (respond)
import CE.Scan.Complexity (eventOf, fold, runs)
import CE.Scan.Cost (scanRowCap)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import Data.Maybe (catMaybes, isJust)
import WireHarness (field, refusedBy, replyObjWith, rowsRequest, runLegs, setKey)

registerPath :: FilePath
registerPath = "../contracts/fixtures/scan/whitepaper.ndjson"

-- | One register line, the columns this battery reads: the margin's
-- cognitive value, its cyclomatic value where the page states one,
-- and the emitter's events (`[seq, parent, pos, flags, aux, op…]`).
data Line = Line
  { lCoc :: Integer
  , lCc :: Maybe Integer
  , lEvents :: [[Integer]]
  }

instance FromJSON Line where
  parseJSON = withObject "line" $ \o -> do
    want <- o .: "want"
    Line <$> want .: "coc" <*> want .:? "cc" <*> o .: "events"

battery :: IO Bool
battery = do
  raw <- B8.readFile registerPath
  let parsed = map decodeStrict (filter (not . B8.null) (map stripCR (B8.lines raw))) :: [Maybe Line]
      register = catMaybes parsed
  runLegs
    (registerNames <> ruleNames <> roadNames <> refusalMessages)
    (registerProbes parsed register <> ruleProbes <> roadProbes register <> refusalProbes)
 where
  stripCR l = if not (B8.null l) && B8.last l == '\r' then B8.init l else l

registerNames :: [String]
registerNames =
  [ "every register line parses"
  , "the register holds the whitepaper's table (thirty lines or more)"
  , "every line's events fold to the page's cognitive value"
  , "every line stating a cyclomatic value folds to it"
  ]

registerProbes :: [Maybe Line] -> [Line] -> [Bool]
registerProbes parsed register =
  [ all isJust parsed
  , length register >= 30
  , and [cocOf (lEvents l) == lCoc l | l <- register]
  , and [maybe True (== ccOf (lEvents l)) (lCc l) | l <- register]
  ]

foldOf :: [[Integer]] -> (Integer, Integer, Integer)
foldOf = fold . map eventOf

cocOf, ccOf :: [[Integer]] -> Integer
cocOf rows = let (_, c, _) = foldOf rows in c
ccOf rows = let (c, _, _) = foldOf rows in c

-- | One event, unit-local: seq, parent (−1 = the unit), pos, flags,
-- aux; a logic root's operator ids follow with `<>`.
ev :: Integer -> Integer -> Integer -> Integer -> Integer -> [Integer]
ev s p pos flags aux = [s, p, pos, flags, aux]

-- | The flag bits as the wire spells them (CE.Scan.Complexity).
nesting, flat, nestOnly, jump, ifKind, chain, ccOp, logicRoot, direct, inAlt :: Integer
nesting = 1
flat = 2
nestOnly = 4
jump = 8
ifKind = 16
chain = 32
ccOp = 128
logicRoot = 256
direct = 512
inAlt = 1024

-- | An if whose first `alternative` child is of class `c` (1 an if,
-- 2 a flat kind, 3 anything else).
ifAlt :: Integer -> Integer
ifAlt c = nesting + ifKind + c * 2048

-- | n nesting structures, each in the previous one's body.
nested :: Integer -> [[Integer]]
nested n = [ev i (i - 1) 1 nesting 0 | i <- [0 .. n - 1]]

ruleNames :: [String]
ruleNames =
  [ "a unit without events reads (1, 0, 0)"
  , "like operators count once per run"
  , "a nesting structure pays one plus its level and raises its body's level, not its header's"
  , "an if under a flat clause is an else-if: one flat point, its body at the chain's level, the clause's own point yielded"
  , "an if in its parent if's alternative field is that if's else-if"
  , "a plain else in the alternative field pays on the if; a flat child pays for itself"
  , "a lambda raises the level and pays nothing"
  , "a labelled jump is a fundamental point at any level"
  , "a chain pays one run and N-1 decisions"
  , "a logic root pays its runs and a decision per operator node"
  ]

ruleProbes :: [Bool]
ruleProbes =
  [ foldOf [] == (1, 0, 0)
  , runs [0, 0, 1, 0, 0] == 3 && runs [] == 0 && runs [1, 1, 1] == 1
  , foldOf (nested 2) == (1, 3, 2) && foldOf [ev 0 (-1) 0 nesting 0, ev 1 0 0 nesting 0] == (1, 2, 1)
  , foldOf elseIfChain == (1, 4, 2) && foldOf plainNest == (1, 6, 3)
  , cocOf [ev 0 (-1) 0 (nesting + ifKind) 0, ev 1 0 1 (nesting + ifKind + direct + inAlt) 0] == 2
      && cocOf [ev 0 (-1) 0 (nesting + ifKind) 0, ev 1 0 1 (nesting + ifKind + direct) 0] == 3
  , cocOf [ev 0 (-1) 0 (ifAlt 3) 0] == 2 && cocOf [ev 0 (-1) 0 (ifAlt 2) 0] == 1
  , foldOf [ev 0 (-1) 0 nestOnly 0, ev 1 0 0 nesting 0] == (1, 2, 2)
  , cocOf [ev 0 (-1) 0 nesting 0, ev 1 0 1 jump 0] == 2 && cocOf [ev 0 (-1) 0 jump 0] == 1
  , foldOf [ev 0 (-1) 0 chain 3] == (3, 1, 0)
  , foldOf [ev 0 (-1) 0 (logicRoot + ccOp) 0 <> [0, 0, 1], ev 1 0 0 (ccOp + direct) 0] == (3, 2, 0)
  ]
 where
  -- if, a flat clause in its alternative, the next if directly under
  -- the clause, and a structure in that if's body
  elseIfChain =
    [ ev 0 (-1) 0 (ifAlt 2) 0
    , ev 1 0 1 (flat + direct + inAlt) 0
    , ev 2 1 1 (nesting + ifKind + direct) 0
    , ev 3 2 1 nesting 0
    ]
  -- the same if nested plainly in the first one's body instead
  plainNest =
    [ ev 0 (-1) 0 (nesting + ifKind) 0
    , ev 2 0 1 (nesting + ifKind + direct) 0
    , ev 3 2 1 nesting 0
    ]

-- | The rows of one unit — cyclomatic, cognitive, nesting — as the
-- client sends them on the events road: zeroed.
triple :: [[Integer]]
triple = [[3, 0], [4, 0], [5, 0]]

-- | A request whose events are keyed at row 1, the cognitive row.
withEvents :: [[Integer]] -> [[Integer]] -> Value
withEvents rows evs = setKey "events" (toJSON (map (1 :) evs)) (rowsRequest "7.0.0" "scan.request" rows)

replyField :: String -> Value -> Maybe Value
replyField k r = replyObjWith respond r >>= (`field` k)

-- | An if over a two-run chain: cyclomatic 4, cognitive 3, depth 1.
sample :: [[Integer]]
sample = [ev 0 (-1) 0 593 0, ev 1 0 0 896 0 <> [0, 1], ev 2 1 0 640 0]

derivedRows :: (Integer, Integer, Integer) -> Value
derivedRows (cc, coc, depth) = toJSON [[0, cc], [1, coc], [2, depth]]

roadNames :: [String]
roadNames =
  [ "the events road derives all three rows and echoes them"
  , "every register line derives on the wire what it folds to"
  , "the levels grade the derived values"
  , "the cycle charge lands on the derived value and cocBumped agrees"
  , "a unit with no events derives (1, 0, 0)"
  , "no events table: the rows keep their bytes and no derived key rides"
  , "an over-cap request with events answers no derived key"
  ]

roadProbes :: [Line] -> [Bool]
roadProbes register =
  [ replyField "derived" (withEvents triple sample) == Just (derivedRows (4, 3, 1))
  , and [replyField "derived" (withEvents triple (lEvents l)) == Just (derivedRows (foldOf (lEvents l))) | l <- register]
  , replyField "levels" (withEvents triple (nested 6)) == Just (toJSON [0, 1, 1 :: Integer])
  , replyField "derived" cycled == Just (derivedRows (4, 4, 1))
      && replyField "cocBumped" cycled == Just (toJSON [[1, 4 :: Integer]])
  , replyField "derived" (withEvents triple []) == Just (derivedRows (1, 0, 0))
  , replyField "levels" legacy == Just (toJSON [0, 0, 0 :: Integer]) && replyField "derived" legacy == Nothing
  , replyField "derived" (withEvents [[0, 0] | _ <- [0 .. scanRowCap]] []) == Nothing
  ]
 where
  cycled = setKey "callEdges" (toJSON [[1, 1 :: Integer]]) (withEvents triple sample)
  legacy = rowsRequest "7.0.0" "scan.request" [[3, 2], [4, 7], [5, 1]]

-- | Every refusal the table can earn, each by the message the core
-- names it with; the requests ride in the parallel list below.
refusalMessages :: [String]
refusalMessages =
  [ "row 1: pre-judged complexity value (events ride)"
  , "row 0: complexity rows must ride as [3,4,5] triples (events ride)"
  , "event 0: row outside the rows"
  , "event 0: row is not a cognitive row"
  , "event 0: negative seq"
  , "event 0: parent not earlier in the unit"
  , "event 0: pos outside 0..1"
  , "event 0: flags outside 13 bits"
  , "event 0: negative aux"
  , "event 0: aux on a non-chain event"
  , "event 0: negative operator"
  , "event 0: logic root without operators"
  , "event 0: operators without a logic root"
  , "event 0: malformed row (need [row,seq,parent,pos,flags,aux,op..])"
  , "event 1: not strictly ascending"
  , "event 1: seq not contiguous"
  ]

refusalProbes :: [Bool]
refusalProbes = zipWith (refusedBy respond) requests refusalMessages
 where
  requests =
    [ withEvents [[3, 0], [4, 7], [5, 0]] []
    , withEvents [[4, 0]] []
    , raw [[9, 0, -1, 0, 1, 0]]
    , raw [[0, 0, -1, 0, 1, 0]]
    , raw [[1, -1, -1, 0, 1, 0]]
    , raw [[1, 0, 0, 0, 1, 0]]
    , raw [[1, 0, -1, 2, 1, 0]]
    , raw [[1, 0, -1, 0, 8192, 0]]
    , raw [[1, 0, -1, 0, 1, -1]]
    , raw [[1, 0, -1, 0, 1, 2]]
    , raw [[1, 0, -1, 0, 256, 0, -1]]
    , raw [[1, 0, -1, 0, 256, 0]]
    , raw [[1, 0, -1, 0, 1, 0, 0]]
    , raw [[1, 0]]
    , raw [[1, 1, 0, 0, 1, 0], [1, 0, -1, 0, 1, 0]]
    , raw [[1, 0, -1, 0, 1, 0], [1, 2, 0, 0, 1, 0]]
    ]
  -- the events as sent, row column included
  raw evs = setKey "events" (toJSON (evs :: [[Integer]])) (rowsRequest "7.0.0" "scan.request" triple)
