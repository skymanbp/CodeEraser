-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The arch.request shape and its boundary contract (design booklet
-- §3 and §7.1; the step-8 brief §1.1): five integer tables — files
-- with their directory and line count, the directory tree with each
-- parent before its child, the file-to-file references, the
-- file-to-directory references, the focus files — each absent read
-- as empty, every row the right width with the right ranges, the
-- reference tables and the focus strictly ascending. The first
-- offender in request order is named as `<table> <i>: <reason>`.
module CE.Arch.Contract (ArchReq (..), offence, overCap) where

import CE.Arch.Cost (fileCap, refCap, rootParent)
import CE.Wire (ascendingOn, rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Foldable (asum)

-- | The request: id and the five tables.
data ArchReq = ArchReq
  { reqId :: Value
  , fileRows :: [[Integer]]
  , dirRows :: [[Integer]]
  , edgeRows :: [[Integer]]
  , pkgRows :: [[Integer]]
  , focusRows :: [Integer]
  }

instance FromJSON ArchReq where
  parseJSON = withObject "ArchReq" $ \o ->
    let table key = o .:? key .!= []
     in ArchReq <$> o .: "id" <*> table "files" <*> table "dirs" <*> table "edges" <*> table "pkgEdges" <*> (o .:? "focus" .!= [])

-- | Two dimensions, two caps: the files, and the two reference
-- tables together (both become arcs of one directory graph).
overCap :: ArchReq -> Bool
overCap req =
  toInteger (length (fileRows req)) > fileCap
    || toInteger (length (edgeRows req) + length (pkgRows req)) > refCap

-- | The first offender in request order. A file row names a
-- directory, so its range check needs the directory table's length
-- only — but a directory table whose rows are malformed has no
-- trustworthy length, so the order is: every file row's own shape,
-- then every directory row's, then the files' directory ranges, then
-- the two reference tables, then the focus.
offence :: ArchReq -> Maybe String
offence req =
  asum
    [ asum (zipWith fileShape [0 ..] (fileRows req))
    , asum (zipWith dirShape [0 ..] (dirRows req))
    , asum (zipWith (fileRange dirs) [0 ..] (fileRows req))
    , tableOffence "edge" (take 2) (edgeShape files) (edgeRows req)
    , tableOffence "pkgEdge" (take 2) (pkgShape files dirs) (pkgRows req)
    , asum (zipWith (focusRange files) [0 ..] (focusRows req))
    , ascendingOn "focus" id (focusRows req)
    ]
 where
  files = toInteger (length (fileRows req))
  dirs = toInteger (length (dirRows req))

-- | [F, D, lines]: F is the row's own index, lines non-negative.
fileShape :: Int -> [Integer] -> Maybe String
fileShape i = rowCheck "file" "malformed file (need [F,D,lines])" 3 checks i
 where
  checks row = case row of
    [f, _, n]
      | f /= toInteger i -> Just ("F must be " <> show i)
      | n < 0 -> Just "negative lines"
    _ -> Nothing

-- | A file's directory is a row of the directory table.
fileRange :: Integer -> Int -> [Integer] -> Maybe String
fileRange dirs i row = case row of
  [_, d, _] | not (inside dirs d) -> Just ("file " <> show i <> ": D out of range")
  _ -> Nothing

-- | [D, parent]: D is the row's own index; row 0 is the root and the
-- only row whose parent is −1; every other parent is an earlier row,
-- so the parent relation is a tree by construction.
dirShape :: Int -> [Integer] -> Maybe String
dirShape i = rowCheck "dir" "malformed dir (need [D,parent])" 2 checks i
 where
  checks row = case row of
    [d, parent]
      | d /= toInteger i -> Just ("D must be " <> show i)
      | i == 0 && parent /= rootParent -> Just "root parent must be -1"
      | i > 0 && parent == rootParent -> Just "second root"
      | i > 0 && not (inside d parent) -> Just "parent out of range"
    _ -> Nothing

-- | [F, G, w]: two files, not one, at least one reference.
edgeShape :: Integer -> Int -> [Integer] -> Maybe String
edgeShape files = rowCheck "edge" "malformed edge (need [F,G,w])" 3 checks
 where
  checks row = case row of
    [f, g, w]
      | not (inside files f && inside files g) -> Just "file out of range"
      | f == g -> Just "self edge"
      | w < 1 -> Just "weight below 1"
    _ -> Nothing

-- | [F, D, w]: a file, a directory, at least one reference.
pkgShape :: Integer -> Integer -> Int -> [Integer] -> Maybe String
pkgShape files dirs = rowCheck "pkgEdge" "malformed pkgEdge (need [F,D,w])" 3 checks
 where
  checks row = case row of
    [f, d, w]
      | not (inside files f) -> Just "file out of range"
      | not (inside dirs d) -> Just "dir out of range"
      | w < 1 -> Just "weight below 1"
    _ -> Nothing

-- | A focus entry names a file.
focusRange :: Integer -> Int -> Integer -> Maybe String
focusRange files i f
  | inside files f = Nothing
  | otherwise = Just ("focus " <> show i <> ": file out of range")

-- | 0 ≤ x < n.
inside :: Integer -> Integer -> Bool
inside n x = x >= 0 && x < n
