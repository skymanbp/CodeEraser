-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | clone.request handler (T3, design vol.2 §4): decode postorder
-- trees, enforce the Cost caps (over-cap = a complete degraded
-- reply, never a truncated one), machine-check the boundary contract
-- in request order (lab/lld lengths, lld range, postorder
-- reconstructibility, non-negative labels, in-range strictly
-- ascending pairs) — then judge: the judge-side admissible prefilter
-- first, Zhang-Shasha TED for the rest. Raw ted and sizes cross the
-- wire, never a ratio — the pre-registered cut table recomputes from
-- one run. The M5-3a stub refused here; this batch replaced exactly
-- that refusal, and the computation lives behind the exhaustive
-- reference harness (core/test/CloneProps.hs ≡ ReferenceTed).
module CE.Clone (respond) where

import CE.Clone.Cost (pairCap, tsedDen, tsedNum, unitNodeCap)
import CE.Clone.Prefilter (provablyBelow)
import CE.Clone.Ted (Tree (..), ted)
import Data.Aeson
import Data.Array.Unboxed (elems, listArray)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

data WireTree = WireTree {wLab :: [Int], wLld :: [Int]}

instance FromJSON WireTree where
  parseJSON = withObject "tree" $ \o -> WireTree <$> o .: "lab" <*> o .: "lld"

data CloneReq = CloneReq
  { reqId :: Value
  , reqTrees :: [WireTree]
  , reqPairs :: [[Int]]
  }

instance FromJSON CloneReq where
  parseJSON = withObject "CloneReq" $ \o ->
    CloneReq <$> o .: "id" <*> o .: "trees" <*> o .: "pairs"

-- | Left = (id to echo, error code, message); Right = the encoded
-- clone.result line.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "clone: " <> e)
  Right req
    | any (\t -> toInteger (length (wLab t)) > unitNodeCap) (reqTrees req)
        || toInteger (length (reqPairs req)) > pairCap ->
        Right (reply proto req [] (0, 0) True)
    | Just why <- violation req -> Left (Just (reqId req), "contract", why)
    | otherwise ->
        let (scores, judged, pre) = judge (map decodeTree (reqTrees req)) (reqPairs req)
         in Right (reply proto req scores (judged, pre) False)

decodeTree :: WireTree -> Tree
decodeTree t =
  Tree
    { tLab = listArray (0, length (wLab t) - 1) (wLab t)
    , tLld = listArray (0, length (wLld t) - 1) (wLld t)
    , tSize = length (wLab t)
    }

-- | First boundary-contract offender in request order (Graph.hs
-- posture: the message names the violator deterministically).
violation :: CloneReq -> Maybe String
violation req =
  asum
    [ asum (zipWith treeShape [0 :: Int ..] ts)
    , asum (zipWith (pairRow (length ts)) [0 :: Int ..] ps)
    , asum (zipWith notAscending [1 :: Int ..] (zip ps (drop 1 ps)))
    ]
 where
  ts = reqTrees req
  ps = reqPairs req

treeShape :: Int -> WireTree -> Maybe String
treeShape t tree
  | null lab = Just (label <> "empty tree")
  | length lab /= length lld = Just (label <> "lab/lld length mismatch")
  | Just i <- badLld = Just (label <> "node " <> show i <> ": lld out of range")
  -- per-node tiling alone admits forests; a single tree's root must
  -- reach the first postorder node
  | last lld /= 0 = Just (label <> "not a single tree (root lld /= 0)")
  | any (< 0) lab = Just (label <> "negative label")
  | Just i <- badSpan = Just (label <> "node " <> show i <> ": children do not tile the span")
  | otherwise = Nothing
 where
  (lab, lld) = (wLab tree, wLld tree)
  label = "tree " <> show t <> ": "
  badLld = lookup True [(l < 0 || l > i, i) | (i, l) <- zip [0 ..] lld]
  badSpan = lookup True [(not (tiles i), i) | i <- [0 .. length lld - 1]]
  -- postorder reconstructibility (F37 replacement): node i's children
  -- must tile [lld i .. i−1] exactly, walking right to left
  tiles i = walk (i - 1)
   where
    low = lld !! i
    walk k
      | k < low = k == low - 1
      | lld !! k < low = False
      | otherwise = walk (lld !! k - 1)

pairRow :: Int -> Int -> [Int] -> Maybe String
pairRow n p row = case row of
  [i, j]
    | i < 0 || j < 0 || i >= n || j >= n -> Just (label <> "endpoint out of range")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [i,j])")
 where
  label = "pair " <> show p <> ": "

notAscending :: Int -> ([Int], [Int]) -> Maybe String
notAscending p (prev, cur)
  | prev < cur = Nothing
  | otherwise = Just ("pair " <> show p <> ": not strictly ascending")

-- | Judge every pair: the admissible prefilter proves "below
-- threshold" without TED where it can; the rest get exact
-- Zhang-Shasha. Score rows carry raw ted and sizes only.
judge :: [Tree] -> [[Int]] -> ([[Integer]], Int, Int)
judge trees ps = foldr step ([], 0, 0) ps
 where
  arr = IM.fromList (zip [0 ..] trees)
  step [i, j] (rows, judged, pre)
    | provablyBelow (elems (tLab a)) (elems (tLab b)) = (rows, judged, pre + 1)
    | otherwise =
        ( [fromIntegral i, fromIntegral j, ted a b, size a, size b] : rows
        , judged + 1
        , pre
        )
   where
    (a, b) = (arr IM.! i, arr IM.! j)
    size = fromIntegral . tSize
  step _ acc = acc -- unreachable: pair shape validated upstream

-- | (judged, prefiltered) travel as the one counts pair they are —
-- six positional parameters was the E01 arity warn (M5 close).
reply :: String -> CloneReq -> [[Integer]] -> (Int, Int) -> Bool -> B8.ByteString
reply proto req scores (judged, pre) degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("clone.result" :: String)
    , "id" .= reqId req
    , "scores" .= scores
    , "counts"
        .= object
          [ "trees" .= length (reqTrees req)
          , "pairs" .= length (reqPairs req)
          , "judged" .= judged
          , "prefiltered" .= pre
          ]
    , "knobs" .= object ["tsedNum" .= tsedNum, "tsedDen" .= tsedDen]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("clone_too_large" :: String) | degraded]
