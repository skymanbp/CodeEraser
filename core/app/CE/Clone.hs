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
-- one run — and since ADR-008 P1 each score row carries the OWNER's
-- verdict bit (Cost.cloneDecides): the reported set is the core's
-- decision, relayed by Rust, never re-derived there. The M5-3a stub
-- refused here; this batch replaced exactly that refusal, and the
-- computation lives behind the exhaustive reference harness
-- (core/test/CloneProps.hs ≡ ReferenceTed).
module CE.Clone (respond) where

import CE.Clone.Cost (cloneDecides, minUnitNodes, pairCap, tsedDen, tsedNum, unitNodeCap)
import CE.Clone.Prefilter (histo, provablyBelowH)
import CE.Clone.Ted (Tree (..), ted)
import CE.Wire (Family (..), respondWith, tableOffence)
import Data.Aeson
import Data.Array.Unboxed (listArray)
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

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "clone"
      , famId = reqId
      , famOverCap = \req ->
          any (\t -> toInteger (length (wLab t)) > unitNodeCap) (reqTrees req)
            || toInteger (length (reqPairs req)) > pairCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req [] (0, 0) True
      , famJudged = \req ->
          let (scores, judged, pre) = judge (map decodeTree (reqTrees req)) (reqPairs req)
           in reply proto req scores (judged, pre) False
      }

decodeTree :: WireTree -> Tree
decodeTree t =
  Tree
    { tLab = listArray (0, length (wLab t) - 1) (wLab t)
    , tLld = listArray (0, length (wLld t) - 1) (wLld t)
    , tSize = length (wLab t)
    , tHisto = histo (wLab t)
    }

-- | First boundary-contract offender in request order (Graph.hs
-- posture: the message names the violator deterministically); the
-- ascending checker is CE.Wire's shared one.
violation :: CloneReq -> Maybe String
violation req =
  asum
    [ asum (zipWith treeShape [0 :: Int ..] ts)
    , tableOffence "pair" id (pairRow (length ts)) ps
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
    -- a unit is not a clone of itself (review C11: [0,0] passed and
    -- judged ted 0 = certain clone)
    | i == j -> Just (label <> "self pair")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [i,j])")
 where
  label = "pair " <> show p <> ": "

-- | Judge every pair: the admissible prefilter proves "below
-- threshold" without TED where it can (below threshold ⇒ not a
-- clone, so no row and no bit); the rest get exact Zhang-Shasha.
-- Score rows carry raw ted and sizes, each paired with the owner's
-- verdict (ADR-008 P1).
judge :: [Tree] -> [[Int]] -> ([([Integer], Bool)], Int, Int)
judge trees ps = foldr step ([], 0, 0) ps
 where
  arr = IM.fromList (zip [0 ..] trees)
  step [i, j] (rows, judged, pre)
    | provablyBelowH (tSize a, tHisto a) (tSize b, tHisto b) = (rows, judged, pre + 1)
    | otherwise =
        ( ( [fromIntegral i, fromIntegral j, d, size a, size b]
          , cloneDecides d (size a) (size b)
          )
            : rows
        , judged + 1
        , pre
        )
   where
    (a, b) = (arr IM.! i, arr IM.! j)
    d = ted a b
    size = fromIntegral . tSize
  step _ acc = acc -- unreachable: pair shape validated upstream

-- | (judged, prefiltered) travel as the one counts pair they are —
-- six positional parameters was the E01 arity warn (M5 close).
-- verdicts is the ADR-008 P1 additive field: one bit per score row,
-- same order — scores stay raw for the instruments' cut tables.
reply :: String -> CloneReq -> [([Integer], Bool)] -> (Int, Int) -> Bool -> B8.ByteString
reply proto req scored (judged, pre) degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("clone.result" :: String)
    , "id" .= reqId req
    , "scores" .= map fst scored
    , "verdicts" .= map snd scored
    , "counts"
        .= object
          [ "trees" .= length (reqTrees req)
          , "pairs" .= length (reqPairs req)
          , "judged" .= judged
          , "prefiltered" .= pre
          ]
    , "knobs"
        .= object
          ["tsedNum" .= tsedNum, "tsedDen" .= tsedDen, "minUnitNodes" .= minUnitNodes]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("clone_too_large" :: String) | degraded]
