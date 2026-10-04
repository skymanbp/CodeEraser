-- | How one group of members that share a hash becomes pairs (attack
-- review D4; plan v2.33 W3): every two members while the group is at
-- most the hot cap, only the adjacent ones above it — a recall bound,
-- counted, never a skipped group (skipping hot groups made detection
-- fall to zero exactly where duplication was highest). The T3 sources
-- S3 and S4 and the docdup coarse filter all pair groups here.
module CE.Candidates.Groups (Paired (..), paired) where

-- | A group's pairs (earlier member first) and whether it was chained.
data Paired = Paired {chained :: Bool, pairsOf :: [(Int, Int)]}

-- | The pairs of one group, members in the order given, under `cap`.
paired :: Int -> [Int] -> Paired
paired cap members
  | length members > cap = Paired True (zip members (drop 1 members))
  | otherwise = Paired False [(a, b) | (a, rest) <- tails' members, b <- rest]
 where
  tails' xs = case xs of
    [] -> []
    (x : more) -> (x, more) : tails' more
