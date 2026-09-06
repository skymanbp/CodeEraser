-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The declaration-relocation battery (7.1.0, O48), written against
-- the contract (booklet 09), not the code: one shared line opens the
-- edge, zero shared lines is the ADAPTED class and opens nothing, two
-- destinations refuse the key, an absent table is not an empty one,
-- the cap refuses the whole table, the floor is exactly the credit,
-- and each boundary offence refuses by its own name.
module DeclProps (battery) where

import CE.FourClass (respond)
import CE.FourClass.Cost (declCredit, declFloor, destFloor, siteCostCross, siteOpens)
import CE.FourClass.Decl (declCap, edges)
import CE.FourClass.Wire (DeclRow, Pair (..))
import Data.Aeson (Value, object, toJSON, (.=))
import qualified Data.Aeson.Key as K
import WireHarness (refusedBy, runChecks)

battery :: IO Bool
battery = runChecks (judgments <> map refusal refusals)

-- | Pairs carrying one leftover line each and the declaration tables
-- under test: `src` loses the key at 10-12 (its line 11), `dst` gains
-- it at 5-7 (its line 6).
pairs :: Word -> [DeclRow] -> [[DeclRow]] -> [Pair]
pairs h rem0 adds =
  Pair 0 [[(11, fromIntegral h, 40)]] [] [] (Just rem0) (Just [])
    : [ Pair i [] [[(6, fromIntegral h, 40)]] [] (Just []) (Just add)
      | (i, add) <- zip [1 ..] adds
      ]

key :: DeclRow
key = (9001, 1, 10, 12)

arrival :: DeclRow
arrival = (9001, 1, 5, 7)

-- | A key whose spans hold no leftover at all: adapted in flight.
adapted :: [Pair]
adapted =
  [ Pair 0 [[(11, 555, 40)]] [] [] (Just [(9002, 2, 20, 21)]) (Just [])
  , Pair 1 [] [[(6, 777, 40)]] [] (Just []) (Just [(9002, 2, 30, 31)])
  ]

judgments :: [(String, Bool)]
judgments =
  [ ("one shared line opens the declaration edge", edges (pairs 555 [key] [[arrival]]) == Just ([(0, 1, 9001)], False))
  , ("two destinations refuse the key", edges (pairs 555 [key] [[arrival], [arrival]]) == Just ([], False))
  , ("zero shared content is adapted and opens nothing", edges adapted == Just ([], False))
  , ("an absent table is not an empty one", edges bare == Nothing && edges empty == Just ([], False))
  , ("over declCap the whole table is refused, not truncated", edges huge == Just ([], True))
  , ("the floor is exactly what the key paid", declFloor == 1 && declCredit == siteCostCross)
  , ("a zero credit collapses the floor onto destFloor", siteOpens siteCostCross destFloor && all (not . siteOpens siteCostCross) [0 .. destFloor - 1])
  ]
 where
  bare = [Pair 0 [] [] [] Nothing Nothing]
  empty = [Pair 0 [] [] [] (Just []) (Just [])]
  huge =
    [ Pair 0 [] [] [] (Just [(fromIntegral n, 1, 1, 1) | n <- [1 .. declCap + 1]]) (Just [])
    , Pair 1 [] [] [] (Just []) (Just [])
    ]

-- | (probe name, the message the boundary must name) — one table, one
-- runner: sibling refusal probes are the shape our own dedup gate
-- flags first.
refusals :: [(Value, String)]
refusals =
  [ (req [obj 3 [("declRem", rows [key])]], "decl tables come in pairs: pair 3")
  , ( req [obj 0 [("declRem", rows [key]), ("declAdd", rows [])], obj 1 []]
    , "decl tables must cover every pair: pair 1"
    )
  , (req [obj 4 [("declRem", rows [(9001, 1, 5, 4)]), ("declAdd", rows [])]], "malformed decl span: pair 4")
  , (req [obj 2 [("declRem", rows [key, key]), ("declAdd", rows [])]], "duplicate decl key: pair 2")
  ]

refusal :: (Value, String) -> (String, Bool)
refusal (r, want) = (want, refusedBy respond r want)

rows :: [DeclRow] -> Value
rows = toJSON

obj :: Int -> [(String, Value)] -> Value
obj i extra =
  object (["i" .= i, "rem" .= ([] :: [Value]), "add" .= ([] :: [Value])] <> [K.fromString k .= v | (k, v) <- extra])

req :: [Value] -> Value
req ps = object ["id" .= (1 :: Int), "pairs" .= ps]
