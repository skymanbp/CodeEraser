-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The erase family against its second implementation (plan v2.32
-- step 7): the verdict pairs, counts and the `kept` closure of the
-- shipped respond must equal ReferenceErase's refusal lists and
-- pairwise closure — on every sequence of up to three rows and every
-- multiset of four over three paths (twin / dead file / verbatim
-- segment, eraseable or not, two spans), on two hundred seeded requests (a fifth of them without the
-- target table, so `kept` must stay absent), on the contract's
-- refusals and on both sides of the row cap.
module EraseEquivProps (battery) where

import CE.Erase (respond)
import CE.Erase.Cost (eraseRowCap)
import Data.Aeson (Value, toJSON)
import Data.List (sortOn)
import ReferenceContract (answers, cells)
import ReferenceErase (expected, exhaustive)
import ReferenceFlowGen (G, S (..), rand, runG)
import WireHarness (rowsRequest, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "every sequence of up to three rows and multiset of four over three paths agrees, kept bits included"
    , "two hundred seeded requests agree, with and without the target table"
    , "the contract's refusals agree with the reference's predicates"
    , "both sides of the row cap agree"
    ]
    [ all (\(ts, rs) -> agrees (rs, [], Just ts)) exhaustive
    , all agrees seeded
    , all agrees refusals
    , all agrees [(replicate n [3, 1, 2, 0, 0], [], Nothing) | n <- [fromInteger eraseRowCap, fromInteger eraseRowCap + 1]]
    ]

request :: [[Integer]] -> [[Integer]] -> Maybe [[Integer]] -> Value
request rows knobs targets =
  maybe id (setKey "targets" . toJSON) targets (setKey "knobs" (toJSON knobs) (rowsRequest "7.0.0" "erase.request" rows))

agrees :: ([[Integer]], [[Integer]], Maybe [[Integer]]) -> Bool
agrees (rows, knobs, targets) =
  answers respond ["rows", "counts", "fail", "degraded", "reason", "kept"] (request rows knobs targets) (expected rows knobs targets)

-- | Two hundred seeded requests: one to nine rows of any live class,
-- every fact drawn inside its legal range, two paths and two spans so
-- targets collide often; one request in five sends no target table.
seeded :: [([[Integer]], [[Integer]], Maybe [[Integer]])]
seeded = [runG one (S (n * 4447 + 3) 0 0 0) | n <- [1 .. 200 :: Int]]
 where
  one = do
    count <- (+ 1) <$> rand 9
    pairs <- sortOn fst <$> mapM (const targetRow) [1 .. count]
    withTargets <- (/= 0) <$> rand 5
    pure (map snd pairs, [], if withTargets then Just (map fst pairs) else Nothing)

targetRow :: G ([Integer], [Integer])
targetRow = do
  cls <- (+ 1) <$> rand 3
  p <- toInteger <$> rand 2
  facts <- mapM rand (ranges cls)
  span' <- rand 2
  let row = toInteger cls : map toInteger facts
      target = if cls == 1 then [p, 1 + 2 * toInteger span', 6] else [p, 0, 0]
  pure (target, if cls == 3 then fixDead row else row)
 where
  ranges cls = case cls of
    1 -> [12, 12, 12, 2]
    2 -> [2, 2, 5, 2]
    _ -> [4, 3, 3, 3]
  fixDead r = case r of
    (c : w : rest) -> c : (w + 1) : rest
    _ -> r

-- | Each row contract, a knob, then each target contract, in the
-- cascade's order (an offending row stands before an offending knob,
-- an offending knob before an offending target).
refusals :: [([[Integer]], [[Integer]], Maybe [[Integer]])]
refusals = [(read rs, read ks, read ts) | [rs, ks, ts] <- cells (unlines table)]
 where
  table =
    [ "[[0,1,1,1,1]] ; [] ; Nothing"
    , "[[4,1,1,1,1]] ; [] ; Nothing"
    , "[[2,1,-1,1,0]] ; [] ; Nothing"
    , "[[3,5,1,0,0]] ; [] ; Nothing"
    , "[[1,9,9,9,2]] ; [] ; Nothing"
    , "[[2,2,1,1,0]] ; [] ; Nothing"
    , "[[2,1,1,5,0]] ; [] ; Nothing"
    , "[[3,1,3,0,0]] ; [] ; Nothing"
    , "[[3,1,2,0]] ; [] ; Nothing"
    , "[[3,1,2,0,0]] ; [[0,1]] ; Just [[0,0,1]]"
    , "[[3,1,2,0,0]] ; [] ; Just []"
    , "[[3,1,2,0,0]] ; [] ; Just [[0,0,1]]"
    , "[[3,1,2,0,0]] ; [] ; Just [[0,3,2]]"
    , "[[1,9,9,9,1]] ; [] ; Just [[0,0,0]]"
    , "[[3,1,2,0,0]] ; [] ; Just [[0,1,4]]"
    , "[[3,1,2,0,0]] ; [] ; Just [[-1,0,0]]"
    , "[[3,1,2,0,0]] ; [] ; Just [[0,0]]"
    , "[[3,1,2,0,0],[3,1,2,0,0]] ; [] ; Just [[1,0,0],[0,0,0]]"
    ]
