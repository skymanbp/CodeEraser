-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The stacking rule's battery (7.0.0, O47): the novel mass that
-- counts is the part INSIDE a newly duplicated unit's span. Written
-- against the contract (booklet 09), not the code: a pure copy fires,
-- the same twenty novel lines placed OUTSIDE every duplicated span do
-- not (the 6.x rule fired on them), the floor is exact, the removal
-- ratio still reads the whole edit, and the boundary refuses a
-- malformed span by name.
module StackingProps (battery) where

import CE.FourClass (respond)
import CE.FourClass.Verdict (stackingNovelFloor, stackingRatio, suspicions)
import CE.FourClass.Wire (Pair (..))
import Data.Aeson (Value (..), encode, object, (.=))
import qualified Data.ByteString.Lazy as BL
import Data.List (isInfixOf)
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("a fresh copy inside a duplicated unit fires", inside == [(0, "stacking")])
    , ("the same novel mass outside every duplicated span is silent", null outside)
    , ("the floor is exact: one line short inside is silent", null short)
    , ("the removal ratio reads the whole edit", null heavy)
    , ("a pair without duplicated spans never fires", null bare)
    , ("a malformed span refuses by name", malformed)
    ]
 where
  pair spans = Pair 0 [] [] spans
  novel = [1 .. stackingNovelFloor]
  inside = suspicions [(pair [(9, 1, stackingNovelFloor)], novel, 0)]
  outside = suspicions [(pair [(9, 100, 200)], novel, 0)]
  short = suspicions [(pair [(9, 1, stackingNovelFloor - 1)], novel, 0)]
  -- floor lines inside the span, deletions exactly at the ratio bar
  heavy = suspicions [(pair [(9, 1, stackingNovelFloor)], novel, stackingNovelFloor `div` stackingRatio)]
  bare = suspicions [(pair [], novel, 0)]
  malformed = case respond "7.0.0" (BL.toStrict (encode req)) of
    Left (_, code, msg) -> code == "contract" && "malformed dup span: pair 4" `isInfixOf` msg
    Right _ -> False
  req =
    object
      [ "id" .= Number 1
      , "pairs" .= [object ["i" .= Number 4, "rem" .= ([] :: [Value]), "add" .= ([] :: [Value]), "dupSpans" .= [[9 :: Int, 5, 4]]]]
      ]
