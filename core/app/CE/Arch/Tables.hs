-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The arch request's path form (plan v2.33 W1 item 3, additive inside
-- the unreleased 9.0.0; the port of cli/src/arch/tables.rs at c2abca5d):
-- the measuring side sends the measured files by path in path order with
-- their line counts, the file-to-file arcs and the section arcs (folded
-- onto their files) as slot pairs with multiplicity, the package arcs as
-- a slot and the package's directory path, and the `--impact` paths; this
-- module builds the tree over the paths (CE.Structure.Tree) and the five
-- integer tables the judgment reads, and keeps each directory's path for
-- the reply. A focus path naming no measured file is a fault answered by
-- name, as the measuring side printed it.
module CE.Arch.Tables (Paths (..), parsePaths, pathsOffence, assemble) where

import CE.Structure.Tree (Dir (..), Tree (..), build, dirOf, dirPaths)
import Data.Aeson (Object, (.!=), (.:), (.:?))
import Data.Aeson.Types (Parser)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | The path form as sent.
data Paths = Paths
  { pPaths :: [String]
  , pLines :: [Integer]
  , pArcs :: [[Integer]]
  , pSections :: [[Integer]]
  , pPackages :: [(Integer, String)]
  , pFocus :: [String]
  }

-- | The path form, when the request carries `paths`.
parsePaths :: Object -> Parser (Maybe Paths)
parsePaths o = do
  paths <- o .:? "paths"
  case paths of
    Nothing -> pure Nothing
    Just ps ->
      fmap Just $
        Paths ps
          <$> o .: "lines"
          <*> o .:? "arcs" .!= []
          <*> o .:? "sections" .!= []
          <*> o .:? "packages" .!= []
          <*> o .:? "focusPaths" .!= []

-- | The path form's own contract: one line count per path (the measuring
-- side's `arch: {} line counts for {} files`), every arc a pair of slots,
-- every package arc a slot.
pathsOffence :: Paths -> Maybe String
pathsOffence p
  | length (pLines p) /= n = Just ("arch: " <> show (length (pLines p)) <> " line counts for " <> show n <> " files")
  | otherwise = case [k | (k, rows) <- [("arcs", pArcs p), ("sections", pSections p)], not (all slotPair rows)] ++ ["packages" | not (all (slot . fst) (pPackages p))] of
      k : _ -> Just (k <> ": a slot outside the paths")
      [] -> Nothing
 where
  n = length (pPaths p)
  slot x = x >= 0 && x < toInteger n
  slotPair row = case row of
    [a, b] -> slot a && slot b
    _ -> False

-- | `assemble`: `[F, D, lines]`, the tree as `[D, parent]` (the root's
-- parent −1), the file arcs and the folded section arcs counted per pair
-- without self pairs, the package arcs counted per (file, directory) — a
-- package outside the tree has no row — and the focus files ascending;
-- with each directory's path (the root ""). Or the focus path the
-- measured files do not hold.
assemble :: Paths -> Either String (([[Integer]], [[Integer]], [[Integer]], [[Integer]], [Integer]), [String])
assemble p = do
  focus <- traverse focusSlot (pFocus p)
  pure ((files, dirs, edges, pkgs, S.toAscList (S.fromList focus)), dirPaths t)
 where
  t = build (pPaths p)
  slots = M.fromList (zip (pPaths p) [0 :: Integer ..])
  files = [[f, maybe 0 toInteger (dirOf t path), n] | (f, path, n) <- zip3 [0 ..] (pPaths p) (pLines p)]
  dirs = [0, -1] : [[toInteger d, toInteger (dParent dir)] | (d, dir) <- drop 1 (IM.toAscList (tDirs t))]
  edges = counted [(a, b) | [a, b] <- pArcs p ++ pSections p, a /= b]
  pkgs = counted [(f, toInteger d) | (f, path) <- pPackages p, Just d <- [M.lookup path (tIds t)]]
  focusSlot path = maybe (Left ("--impact: " <> path <> " is not a measured file")) Right (M.lookup path slots)

-- | `counted`: one `[a, b, w]` row per distinct pair, ascending.
counted :: [(Integer, Integer)] -> [[Integer]]
counted pairs = [[a, b, w] | ((a, b), w) <- M.toAscList (M.fromListWith (+) [(pr, 1) | pr <- pairs])]
