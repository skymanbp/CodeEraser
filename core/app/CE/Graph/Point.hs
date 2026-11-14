-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The dead-file comparison point and the gate over it (plan v2.35,
-- the dead-file amendment, phase 5B). The measuring side names every
-- file node by an entity id (`entities` [node, id]: the id its size row
-- has, a fact it read, never a judgment) and carries the committed
-- baseline's `deadPoint` verbatim: `files`, the ids of every file node
-- the graph judged on the day the baseline was written, and `dead`, the
-- ids of those that were unreferenced, both ascending, `dead` a subset
-- of `files`. Let D be the unreferenced files (the `dead` rows). EVERY
-- member of D has a STATE, roots and dependents alike, by
-- CE.Graph.Cost.rootStates: no point -> untold; its id in the point's
-- `dead` -> standing; in `files` and not in `dead` -> orphaned; not in
-- `files` -> born. `deadBlock` is the project's mask over the blockable
-- states (CE.Graph.Cost.blockStates, bit i = the i-th); M is the members
-- of D whose state's bit is in the mask, and a file BLOCKS when it is a
-- root OF M: in M, and supported by every member of M that supports it
-- (CE.Graph.Roots' law with M in the place of D; the caller hands that
-- call in, the pairs never forced). The gate passes iff M is empty: a
-- non-empty finite M has a root. When M = D the blocked set is D's
-- roots; every root of D in M is a root of M. A mask without a point is
-- UNANSWERED and blocks nothing (every state is untold, M is empty).
-- `deadNow` is today's point, the object the next baseline records (the
-- verdict family copies it verbatim).
module CE.Graph.Point (Gate (..), addedOffence, blockOffence, entitiesOffence, gate, pointOffence, pointShape, pointSize, readPoint) where

import CE.Graph.Cost (blockStates, granFile, rootStates)
import CE.Wire (label, u64)
import Data.Aeson
import qualified Data.Aeson.Types as AT
import Data.Aeson.Types (Pair)
import Data.Bits (testBit)
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import Data.List (sort)
import qualified Data.Set as S

-- | The point's two lists, read and held to their shape: each strictly
-- ascending inside u64, `dead` inside `files`.
readPoint :: Value -> Either String ([Integer], [Integer])
readPoint v = case AT.parse (withObject "deadPoint" (\o -> (,) <$> o .: "files" <*> o .: "dead")) v of
  AT.Error e -> Left ("deadPoint: " <> e)
  AT.Success (fs, ds) -> maybe (Right (fs, ds)) Left (asum [ids "deadPoint.files" fs, ids "deadPoint.dead" ds, subset fs ds])
 where
  ids what xs =
    asum
      ( [Just (what <> ": element outside u64") | any (not . u64) xs]
          <> [Just (label what i <> "not strictly ascending") | (i, (a, b)) <- zip [1 :: Int ..] (zip xs (drop 1 xs)), a >= b]
      )
  subset fs ds
    | S.fromList ds `S.isSubsetOf` S.fromList fs = Nothing
    | otherwise = Just "deadPoint.dead: not a subset of files"

-- | The elements a point carries, priced before its shape is read (the
-- family's cap disjunct): the longer of its two lists, 0 off shape.
pointSize :: Maybe Value -> Int
pointSize p = case p of
  Just (Object o) -> maximum (0 : [length a | Just (Array a) <- map (`lookupKey` o) ["files", "dead"]])
  _ -> 0
 where
  lookupKey k o = AT.parseMaybe (.: k) o

-- | A point that rode is well shaped and rides beside its entities.
pointOffence :: Maybe [[Integer]] -> Maybe Value -> Maybe String
pointOffence ents p = case (p, ents) of
  (Just _, Nothing) -> Just "deadPoint: entities table required alongside"
  _ -> pointShape p

-- | A point that rode is well shaped (the verdict family's check: its
-- ids are the file rows' identities already).
pointShape :: Maybe Value -> Maybe String
pointShape = maybe Nothing (either Just (const Nothing) . readPoint)

-- | The mask names blockable states only (bit i = blockStates !! i).
blockOffence :: Maybe Integer -> Maybe String
blockOffence m = case m of
  Just b | b < 0 || b >= 2 ^ length blockStates -> Just ("deadBlock: a bit outside the blockable states (" <> unwords blockStates <> ")")
  _ -> Nothing

-- | The entity table: [node, id] rows ascending by node, one per FILE
-- node and none for another kind, every id inside u64 and no id twice.
entitiesOffence :: [[Integer]] -> Maybe [[Integer]] -> Maybe String
entitiesOffence nodes ents = case ents of
  Nothing -> Nothing
  Just rows ->
    asum
      [ asum [Just (label "entities" i <> "malformed row (need [node,id])") | (i, r) <- zip [0 :: Int ..] rows, length r /= 2]
      , asum [Just (label "entities" i <> "node out of range") | (i, n : _) <- zip [0 :: Int ..] rows, n < 0 || n >= toInteger (length nodes)]
      , asum [Just (label "entities" i <> "id outside u64") | (i, [_, x]) <- zip [0 :: Int ..] rows, not (u64 x)]
      , asum [Just (label "entities" i <> "not strictly ascending") | (i, (a : _, b : _)) <- zip [1 :: Int ..] (zip rows (drop 1 rows)), a >= b]
      , asum [Just (label "entities" i <> "node " <> show n <> " is not a file node") | (i, n : _) <- zip [0 :: Int ..] rows, S.notMember n files]
      , asum [Just ("entities: file node " <> show n <> " without a row") | n <- S.toList (files `S.difference` S.fromList [n | n : _ <- rows])]
      , asum [Just ("entities: two nodes with id " <> show x) | (x, k) <- M.toList (M.fromListWith (+) [(x, 1 :: Int) | [_, x] <- rows]), k > 1]
      ]
 where
  files = S.fromList [i | (i, _ : k : _) <- zip [0 ..] nodes, k == granFile]

-- | The verdict family's `deadAdded` (the blocking files' ids): it
-- rides beside the point it was judged against, strictly ascending,
-- inside the point's `dead`.
addedOffence :: Maybe Value -> Maybe [Integer] -> Maybe String
addedOffence p added = case (added, p >>= either (const Nothing) Just . readPoint) of
  (Nothing, _) -> Nothing
  (Just _, Nothing) -> Just "deadAdded: deadPoint required alongside"
  (Just xs, Just (_, ds))
    | or (zipWith (>=) xs (drop 1 xs)) -> Just "deadAdded: not strictly ascending"
    | not (S.fromList xs `S.isSubsetOf` S.fromList ds) -> Just "deadAdded: an id outside deadPoint.dead"
    | otherwise -> Nothing

-- | One judgment's gate: every unreferenced file's state code, the
-- blocked files and the reply keys (`states` [node, state] one row per
-- unreferenced file ascending, `blocked`, `blockUnanswered` when true,
-- `deadNow` when the entities rode). The contract has held every
-- input's shape. `rootsOf` answers the roots of a set of unreferenced
-- files (CE.Graph.Roots' law over that set).
data Gate = Gate {gState :: Int -> Integer, gBlocked :: [Int], gKeys :: [Pair]}

gate :: Maybe [[Integer]] -> Maybe Value -> Integer -> [Int] -> ([Int] -> [Int]) -> Gate
gate ents point mask deads rootsOf = Gate stateOf blocked keys
 where
  idOf = IM.fromList [(fromInteger n, x) | [n, x] <- maybe [] id ents]
  told = either (const Nothing) (\(fs, ds) -> Just (S.fromList fs, S.fromList ds)) . readPoint =<< point
  stateOf n = code $ case (told, IM.lookup n idOf) of
    (Just (fs, ds), Just x)
      | S.member x ds -> "standing"
      | S.member x fs -> "orphaned"
      | otherwise -> "born"
    _ -> "untold"
  -- M, and the files that block: M's roots (none when M is empty)
  masked = [d | d <- deads, Just b <- [lookup (stateOf d) blockable], testBit mask b]
  blocked = if null masked then [] else rootsOf masked
  blockable = [(code s, b) | (b, s) <- zip [0 ..] blockStates]
  keys =
    ["states" .= [[toInteger d, stateOf d] | d <- deads], "blocked" .= blocked]
      <> ["blockUnanswered" .= True | mask /= 0, Nothing <- [told]]
      <> ["deadNow" .= object ["files" .= sort (IM.elems idOf), "dead" .= sort [x | d <- deads, Just x <- [IM.lookup d idOf]]] | Just _ <- [ents]]
  code s = toInteger (length (takeWhile (/= s) rootStates))
