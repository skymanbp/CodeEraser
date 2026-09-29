-- | The scan request's events table (7.2.0, plan v2.30 step 7b ③):
-- its boundary contract, the derivation that writes each unit's
-- three complexity rows from its events, and the echo that carries
-- them back. Each row is `[row, seq, parent, pos, flags, aux, op…]`:
-- `row` the unit's cognitive row, whose cyclomatic and nesting rows
-- flank it (the client's block layout, verified here and never
-- assumed); `seq` the unit-local pre-order index, contiguous from 0;
-- `parent` −1 or an earlier seq; `pos` 0 or 1; `flags` thirteen bits;
-- `aux` a chain's operand count and 0 elsewhere; the tail a logic
-- root's operator ids, present exactly when that bit is. When the
-- table rides, every code-3, -4 and -5 row must carry 0 — the value
-- is derived here, by the judgment's owner, the way the naming facts
-- settle the code-6 rows (the staleDocs lesson: one judgment, one
-- road) — and every such row must sit in a [3,4,5] triple. A unit
-- without events is derived all the same: (1, 0, 0).
--
-- Every message here is golden-pinned text; a refusal names the table,
-- the row and the reason.
module CE.Scan.Events (derivedRows, eventBattery, withEvents) where

import CE.Scan.Complexity (eventOf, fold)
import CE.Wire (ascendingOn)
import Data.Bits (testBit)
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.List (find)
import qualified Data.Map.Strict as M

-- | The codes this table derives (CE.Scan.Cost's frozen positions):
-- cyclomatic, cognitive, nesting — consecutive in a unit's block.
derived :: [Integer]
derived = [3, 4, 5]

-- | First offender in request order: the rows' own posture on the
-- events road (zeroed, in triples), then each event's shape, then
-- the order and the seq contiguity across the table.
eventBattery :: [[Integer]] -> Maybe [[Integer]] -> Maybe String
eventBattery _ Nothing = Nothing
eventBattery rows (Just evs) =
  asum
    [ asum (zipWith rowPosture [0 :: Int ..] rows)
    , asum (zipWith (eventShape n codeAt) [0 :: Int ..] evs)
    , ascendingOn "event" (take 2) evs
    , contiguous evs
    ]
 where
  -- the rows' codes indexed once: the posture check reads each row's
  -- neighbours and every event reads its row, and walking the list
  -- from its head for each of them was quadratic in the row count
  -- (2.4 s of the self tree's scan, 35k rows)
  codes = IM.fromList [(i, c) | (i, c : _) <- zip [0 ..] rows]
  n = toInteger (length rows)
  codeAt i = IM.lookup i codes
  rowPosture i row = case row of
    [c, v]
      | c `elem` derived && v /= 0 -> Just (label "row" i <> "pre-judged complexity value (events ride)")
      | c == 3 && codeAt (i + 1) /= Just 4 -> Just (label "row" i <> triple)
      | c == 5 && codeAt (i - 1) /= Just 4 -> Just (label "row" i <> triple)
      | c == 4 && (codeAt (i - 1) /= Just 3 || codeAt (i + 1) /= Just 5) -> Just (label "row" i <> triple)
    _ -> Nothing
  triple = "complexity rows must ride as [3,4,5] triples (events ride)"

-- | One event's shape, by name, against the row count and the code
-- index; the width check first so every later read is on a row that
-- has the column.
eventShape :: Integer -> (Int -> Maybe Integer) -> Int -> [Integer] -> Maybe String
eventShape n codeAt i row = case row of
  (r : s : p : pos : flags : aux : ops) ->
    fmap (label "event" i <>) $
      snd
        <$> find
          fst
          [ (r < 0 || r >= n, "row outside the rows")
          , (codeAt (fromInteger r) /= Just 4, "row is not a cognitive row")
          , (s < 0, "negative seq")
          , (p < -1 || p >= s, "parent not earlier in the unit")
          , (pos < 0 || pos > 1, "pos outside 0..1")
          , (flags < 0 || flags >= 8192, "flags outside 13 bits")
          , (aux < 0, "negative aux")
          , (aux /= 0 && not (testBit flags 5), "aux on a non-chain event")
          , (any (< 0) ops, "negative operator")
          , (null ops && testBit flags 8, "logic root without operators")
          , (not (null ops) && not (testBit flags 8), "operators without a logic root")
          ]
  _ -> Just (label "event" i <> "malformed row (need [row,seq,parent,pos,flags,aux,op..])")

-- | Within one unit the seqs are 0, 1, 2, … in table order: the
-- k-th event of a row carries seq k.
contiguous :: [[Integer]] -> Maybe String
contiguous evs = go M.empty (zip [0 :: Int ..] evs)
 where
  go _ [] = Nothing
  go seen ((i, r : s : _) : rest)
    | s /= M.findWithDefault 0 r seen = Just (label "event" i <> "seq not contiguous")
    | otherwise = go (M.insert r (s + 1) seen) rest
  go seen (_ : rest) = go seen rest

label :: String -> Int -> String
label what i = what <> " " <> show i <> ": "

-- | The rows with each unit's three complexity values derived from
-- its events — the table absent, the rows keep their bytes.
withEvents :: Maybe [[Integer]] -> [[Integer]] -> [[Integer]]
withEvents Nothing rows = rows
withEvents (Just evs) rows = zipWith settle [0 :: Int ..] rows
 where
  -- fromListWith prepends, so each unit's list comes out reversed
  -- and is turned back: linear, and the order is the table's
  units = M.map reverse (M.fromListWith (++) [(fromInteger r, [eventOf rest]) | (r : rest) <- evs])
  folded = M.map fold units
  numbers i = M.findWithDefault (fold []) i folded
  settle i row = case row of
    [3, _] -> let (cc, _, _) = numbers (i + 1) in [3, cc]
    [4, _] -> let (_, coc, _) = numbers i in [4, coc]
    [5, _] -> let (_, _, depth) = numbers (i - 1) in [5, depth]
    _ -> row

-- | The echo: every derived row as judged, `[rowIndex, value]` in
-- row order — the measuring side renders the numbers without ever
-- deriving one for itself.
derivedRows :: [[Integer]] -> [[Integer]]
derivedRows rows = [[i, v] | (i, [c, v]) <- zip [0 ..] rows, c `elem` derived]
