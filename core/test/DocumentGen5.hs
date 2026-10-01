-- | The seeded requests of step 5's two sentence families (plan v2.32
-- step 5): two hundred guard requests — one to four rules speaking,
-- each rule's integers drawn, the probe matches and tombstone places
-- they slice from fixed pools — and two hundred audit requests over
-- the three faces, every leg present or absent, judged or degraded,
-- whole or partial; drawn by the flow reference's generator, as
-- DocumentGen and DocumentGen4 draw the document families'.
module DocumentGen5 (requests5) where

import Control.Monad (forM, replicateM)
import Data.Aeson (Value)
import DocumentGen (docRequest, num)
import ReferenceFlowGen (G, S (..), rand, runG)

requests5 :: String -> [Value]
requests5 fam = [runG (if fam == "guard" then guard else audit) (S (n * 6151 + 29) 0 0 0) | n <- [1 .. 200]]

guard :: G Value
guard = do
  n <- (+ 1) <$> rand 4
  say <- replicateM n (rand 6 >>= rule)
  matches <- forM [0 .. 5] (\k -> (\s l t -> [k, s + 1, s + 1 + l, t + 50]) <$> num 400 <*> num 30 <*> num 300)
  places <- pool 6
  pure $
    docRequest
      "guard"
      [("files", 3), ("matches", 6), ("places", 6), ("units", 2), ("errors", 1)]
      [("say", say), ("matches", matches), ("places", places)]
      []
      Nothing
 where
  -- a slice [first, count] of a pool of six, three shown at most
  slice = do
    a <- num 4
    c <- num 3
    pure (a, min c (6 - a))
  rule :: Int -> G [Integer]
  rule r = case r of
    0 -> (\f g (a, c) -> [0, f, g + 1, a, c + 1, 0, 0]) <$> num 3 <*> num 9 <*> slice
    1 -> (\f l x fence -> [1, f, 751 + l, 750 + x, fence, 0, 0]) <$> num 3 <*> num 900 <*> num 2 <*> num 3
    2 -> (\f l p -> [2, f, 300 + l, p, 300, 750, 0]) <$> num 3 <*> num 450 <*> num 1000
    3 -> (\s b fence (a, c) -> [3, s + 1, b, fence, a, c + 1, 0]) <$> num 9 <*> num 4 <*> num 3 <*> slice
    4 -> (\f v u k l -> [4, f, v + 1, u, k, l + 1, 0]) <$> num 3 <*> num 4 <*> num 2 <*> num 4 <*> num 300
    _ -> pure [5, 0, 0, 0, 0, 0, 0]

-- | Tombstone places [i, line, kind], one per slot.
pool :: Integer -> G [[Integer]]
pool n = forM [0 .. n - 1] (\i -> (\l k -> [i, l + 1, k]) <$> num 200 <*> num 3)

audit :: G Value
audit = do
  face <- num 3
  git <- (\x -> if x == 0 then 0 else 1) <$> rand 6
  unreadable <- if face == 2 then (\x -> if x == 0 then 1 else 0) <$> rand 6 else pure 0
  mounted <- if face == 0 then num 2 else pure 0
  mode <- num 4
  changed <- num 9
  net <- (\x -> [[x - 40]]) <$> num 80
  dups <- rand 4 >>= \x -> if x /= 0 then (\c f -> [[c, f]]) <$> num 4 <*> num 2 else pure []
  blocks <- forM [0 .. 2] (\k -> (\s l t -> [k, s + 1, s + 1 + l, s + 9, s + 9 + l, t + 50]) <$> num 300 <*> num 20 <*> num 200)
  tomb <- rand 4 >>= \x -> if x /= 0 then (: []) <$> leg else pure []
  places <- pool 3
  pure $
    docRequest
      "audit"
      [("blocks", 3), ("places", 3), ("errors", 1)]
      [("net", net), ("dups", dups), ("blocks", blocks), ("tomb", tomb), ("places", places)]
      (zip (words "face git unreadable mounted mode changed") [face, git, unreadable, mounted, mode, changed])
      Nothing
 where
  -- [state, sites, label, prose, erased, budget, tier, over, unread, bounded]
  leg = do
    state <- (+ 1) <$> num 2
    sites <- num 4
    label <- num (fromInteger sites + 1)
    erased <- num 6
    budget <- num 3
    tier <- (\x -> if x < 2 then 3 else x - 2) <$> num 5
    over <- num 2
    unread <- (\x -> if x == 0 then 1 else 0) <$> rand 3
    bounded <- (\x -> if x == 0 then 2 else 0) <$> rand 4
    pure [state, sites, label, sites - label, erased, budget, tier, over, unread, bounded]
