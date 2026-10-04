-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The L1 judgment (moves/1, plan v2.33 W3): one hand-worked pair
-- pins every field (a move, a blank that never moves, a run the move
-- closes), two hundred seeded changesets agree with ReferenceMoves, the
-- seeded changesets are not vacuous, the refusals name their offender,
-- and the line and unit ceilings hold.
module MovesProps (battery) where

import CE.FourClass.Moves (respond)
import CE.FourClass.Moves.Contract (MovesReq (..), PairIn (..), Side (..), overCap)
import CE.FourClass.Moves.Cost (movesLineCap, movesUnitCap)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import ReferenceMoves (Pair, movesExpected, movesRequest, movesRequests)
import WireHarness (fieldsOf, refusedBy, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "a hand-worked pair: a move, a blank that never moves, a run the move closes"
    , "two hundred seeded changesets agree with the reference"
    , "the seeded changesets are not vacuous: moves, relocations, declarations, duplicates, bridged runs"
    , "the refusals name their offender"
    , "the line and unit ceilings degrade, never truncate"
    ]
    [ handWorked
    , all agrees movesRequests
    , nonVacuous
    , all (uncurry (refusedBy respond)) refusals
    , caps
    ]

pairsOf :: Value -> Maybe [Maybe Value]
pairsOf r = fieldsOf respond r ["pairs"]

-- | Before [t1, blank, t2], after [t2, blank, t3], every line changed,
-- one unit spanning each side: t2 moved both ways, t1 deleted, t3
-- novel, the blanks plain; nothing relocated (the unit's other changed
-- lines are no moves), nothing one-sided, no duplicate; the before run
-- holds t1 alone (the move closes it), the after run t3 alone.
handWorked :: Bool
handWorked =
  pairsOf (movesRequest [(([1, 0, 2], [0, 1, 2], [unit]), ([2, 0, 3], [0, 1, 2], [unit]))])
    == Just [Just (toJSON [want])]
 where
  unit = [0, 0, 1, 3, 1]
  want =
    object
      [ "counts" .= [2, 1, 2, 1 :: Int]
      , "moved" .= [[3, 1, 0], [1, 0, 0 :: Int]]
      , "relocated" .= ([] :: [Int])
      , "declRem" .= ([] :: [Int])
      , "declAdd" .= ([] :: [Int])
      , "dupSpans" .= ([] :: [[Int]])
      , "runsRem" .= [[[1, 2654435768, 2 :: Integer]]]
      , "runsAdd" .= [[[3, 7963307290, 1 :: Integer]]]
      ]

agrees :: [Pair] -> Bool
agrees ps = pairsOf (movesRequest ps) == Just [Just (movesExpected ps)]

-- | Some seeded changeset shows each field non-empty.
nonVacuous :: Bool
nonVacuous = all shown ["moved", "relocated", "declRem", "declAdd", "dupSpans"] && bridged
 where
  answers = concat [toList xs | Array xs <- map movesExpected movesRequests]
  shown k = any (\a -> nonEmpty (fieldAt k a)) answers
  fieldAt k (Object o) = KM.lookup k o
  fieldAt _ _ = Nothing
  nonEmpty (Just (Array xs)) = not (null xs)
  nonEmpty _ = False
  -- a run of two entries whose lines are not adjacent: a blank bridge
  bridged = or [gap r | a <- answers, Just (Array rs) <- [fieldAt "runsRem" a], Array r <- toList rs]
  gap r = case [l | Array e <- toList r, (Number l : _) <- [toList e]] of
    (x : y : _) -> y - x > 1
    _ -> False

-- | A raw request: the keys, then each pair's two sides as (lines,
-- changed, units).
raw :: [Integer] -> [([[Integer]], [Integer], [[Integer]])] -> Value
raw keys sides =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("moves.request" :: String)
    , "id" .= (1 :: Int)
    , "keys" .= keys
    , "pairs" .= [object ["before" .= side b, "after" .= side a] | (b, a) <- byTwo sides]
    ]
 where
  side (ls, cs, us) = object ["lines" .= ls, "changed" .= cs, "units" .= us]
  byTwo (b : a : rest) = (b, a) : byTwo rest
  byTwo _ = []

-- | Each refusal: the request, the offender's stem.
refusals :: [(Value, String)]
refusals =
  [ (raw [-1] [], "key 0: outside u64")
  , (raw [] [([[0, 1, -1]], [], []), none], "pair 0 before line 0: negative width")
  , (raw [] [([[-1, 1, 1]], [], []), none], "pair 0 before line 0: negative content code")
  , (raw [] [([[1, 1]], [], []), none], "pair 0 before line 0: malformed line (need [code,hash,width])")
  , (raw [] [none, ([[0, 1, 1]], [1], [])], "pair 0 after changed 0: out of range")
  , (raw [] [([[0, 1, 1], [1, 2, 1]], [1, 0], []), none], "pair 0 before changed 1:")
  , (raw [7] [none, ([[0, 1, 1]], [], [[1, 0, 1, 1, 0]])], "pair 0 after unit 0: key outside the key table")
  , (raw [7] [([[0, 1, 1]], [], [[0, 0, 2, 1, 0]]), none], "pair 0 before unit 0: span not 1-based and ordered")
  , (raw [7] [none, none, ([[0, 1, 1]], [], [[0, 0, 1, 1, 2]]), none], "pair 1 before unit 0: stack not a bit")
  , (raw [7] [none, ([[0, 1, 1]], [], [[0, -1, 1, 1, 0]])], "pair 0 after unit 0: negative kind")
  ]
 where
  none = ([], [], [])

-- | One past each ceiling is over it, at each it is not — counted on
-- the contract over shared rows, not over a four-million-row request.
caps :: Bool
caps =
  not (overCap (req lineRows 0)) && overCap (req (lineRows <> [[0, 0, 0]]) 0)
    && not (overCap (req [] 0)) && overCap (req [] 1)
 where
  lineRows = replicate (fromInteger movesLineCap) [0, 0, 0]
  unitRows extra = replicate (fromInteger movesUnitCap + extra) [0, 0, 1, 1, 0]
  req ls extra = MovesReq Null [0] [PairIn (Side ls (unitRows extra) []) (Side [] [] [])]
