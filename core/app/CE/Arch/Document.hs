-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce arch` document (plan v2.32 step 3; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/arch/face.rs): the
-- layers, the cut arcs with the file references each one folds, the
-- clusters with their majority directory, the misplaced files, the
-- impact of the focus and the per-directory metrics, every path a
-- reference. The request sends back the arch/1 request and answer
-- tables and two rank tables: each file path's and each slashed
-- directory's place in one string order, so the folded references
-- sort here exactly as the strings would.
module CE.Arch.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.List (sortOn)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = docFamily "arch" schemaId statement slots assemble []

schemaId :: String
schemaId = "ce.arch-report/0.1.0"

-- | The request (a `files` row is arch/1's [F, D, lines], a rank row
-- [slot, place in the joint order of file paths and slashed
-- directories, the root `./`], a `widths` row a directory path's [D,
-- bytes, characters]: the console pads the metrics table to the
-- widest path, plan v2.32 step 5).
-- References: path [file], dir [directory], slashed [directory], why [text].
statement :: String
statement =
  "range files\nrange dirs\nrange why\n\
  \rows files 3 judged files dirs -\nrows dirs 2 judged dirs -\n\
  \rows edges 3 judged files files -\nrows pkgEdges 3 judged files dirs -\n\
  \rows focus 1 judged files\nrows layers 2 judged dirs -\n\
  \rows cuts 4 judged dirs dirs - -\nrows clusters 2 judged files -\n\
  \rows misplaced 2 judged files dirs\nrows impact 2 judged files -\n\
  \rows metrics 4 judged dirs - - -\n\
  \rows rankFiles 2 judged files -\nrows rankDirs 2 judged dirs -\nrows widths 3 judged dirs - -\noptional widths\n\
  \ref path files\nref dir dirs\nref slashed dirs\nref why why\n"

-- | The tables read by slot hold one row per slot (a degraded request
-- holds none; an absent optional table is not read by slot).
slots :: DocReq -> Maybe String
slots req
  | Just _ <- dDegraded req = Nothing
  | otherwise = asum [dense req t (range req u) | (t, u) <- perSlot, maybe False (M.member t) (dRows req)]
 where
  perSlot = [("files", "files"), ("dirs", "dirs"), ("layers", "dirs"), ("metrics", "dirs"), ("rankFiles", "files"), ("rankDirs", "dirs"), ("widths", "dirs")]


assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= schemaId
    , "counts" .= counted countKeys tallies
    , "layers" .= [object ["dir" .= dir d, "level" .= l] | [d, l] <- rows req "layers"]
    , "cuts" .= map (cut (folded req dirAt)) (rows req "cuts")
    , "clusters" .= [cluster dirAt c fs | (c, fs) <- M.toAscList clustered]
    , "misplaced" .= [object ["path" .= path f, "dir" .= dir (dirAt f), "majority" .= dir m] | [f, m] <- rows req "misplaced"]
    , "impact" .= [object ["path" .= path f, "depth" .= d] | [f, d] <- rows req "impact"]
    , "metrics" .= map metric (rows req "metrics")
    , "degraded" .= whyRef req
    ]
 where
  dirAt = lookupIn (rows req "files")
  clustered = M.fromListWith (flip (<>)) [(c, [f]) | [f, c] <- rows req "clusters"]
  countKeys = words "files dirs edges pkgEdges focus cuts clusters misplaced impact"
  sized = map (toInteger . length . rows req)
  tallies = sized ["files", "dirs", "edges", "pkgEdges", "focus", "cuts"] <> [toInteger (M.size clustered)] <> sized ["misplaced", "impact"]
  metric r = case r of
    [d, fanIn, fanOut, s] -> object ["dir" .= dir d, "fanIn" .= fanIn, "fanOut" .= fanOut, "instability" .= (if s >= 0 then toJSON s else Null)]
    _ -> Null

path, dir, slashed :: Integer -> Value
path f = ref "path" [f]
dir d = ref "dir" [d]
slashed d = ref "slashed" [d]

-- | A slot's second column off a table keyed by its first (a file's
-- directory, a path's rank); −1 off the table.
lookupIn :: [[Integer]] -> Integer -> Integer
lookupIn table = \x -> IM.findWithDefault (-1) (fromInteger x) m
 where
  m = IM.fromList [(fromInteger s, v) | s : v : _ <- table]

-- | Every file reference keyed by the directory arc it folds onto —
-- the file edges first, then the package references — each with its
-- (from, to) place in string order.
folded :: DocReq -> (Integer -> Integer) -> M.Map (Integer, Integer) [((Integer, Integer), Value)]
folded req dirAt = M.fromListWith (flip (<>)) (fileRefs <> pkgRefs)
 where
  (byFile, byDir) = (lookupIn (rows req "rankFiles"), lookupIn (rows req "rankDirs"))
  reference f to w = object ["from" .= path f, "to" .= to, "refs" .= w]
  fileRefs = [((dirAt f, dirAt g), [((byFile f, byFile g), reference f (path g) w)]) | [f, g, w] <- rows req "edges"]
  pkgRefs = [((dirAt f, d), [((byFile f, byDir d), reference f (slashed d) w)]) | [f, d, w] <- rows req "pkgEdges"]

-- | One cut arc with the references it folds, ordered by (from, to)
-- as strings — by rank, ties (none: the pairs are distinct) kept in
-- order.
cut :: M.Map (Integer, Integer) [((Integer, Integer), Value)] -> [Integer] -> Value
cut refs row = case row of
  [a, b, w, e] ->
    object
      [ "from" .= dir a
      , "to" .= dir b
      , "refs" .= w
      , "exact" .= (e == 1)
      , "files" .= map snd (sortOn fst (M.findWithDefault [] (a, b) refs))
      ]
  _ -> Null

-- | A cluster with its files in row order and the directory holding
-- most of them — the least directory on a tie, the reading the core
-- names a misplaced file's majority by.
cluster :: (Integer -> Integer) -> Integer -> [Integer] -> Value
cluster dirAt c fs = object ["cluster" .= c, "majority" .= dir majority, "files" .= map path fs]
 where
  held = M.fromListWith (+) [(dirAt f, 1 :: Int) | f <- fs]
  top = maximum (M.elems held)
  majority = minimum [d | (d, n) <- M.toList held, n == top]
