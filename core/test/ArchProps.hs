-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The arch family's battery (plan v2.31 step 8): the hand-written
-- cases of ArchCases and ArchRefusals, one leg each, the feedback arc
-- set legs of ArchFasProps, then the clustering's determinism and its
-- modularity against the all-singleton partition, the impact against
-- the fixpoint closure, the caps, the empty request and the counts.
module ArchProps (battery) where

import ArchCases (Case (..), judgments)
import ArchFasProps (draw, fasLegs, ordered, stream)
import ArchRefusals (refusals)
import CE.Arch (respond)
import CE.Arch.Contract (ArchReq (..), overCap)
import CE.Arch.Cost (fileCap, refCap)
import CE.Arch.Dirs (fileNeighbours)
import CE.Arch.Louvain (clusters)
import Data.Aeson (Result (..), Value (..), encode, fromJSON, object, toJSON, (.=))
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntMap.Strict as IM
import Data.List (isPrefixOf)
import ReferenceArch (closure, modularity4m2)
import WireHarness (fieldsOf, refusedBy, runLegs, setKey, tabledRequest)

battery :: IO Bool
battery = do
  (fasNames, fasProbes) <- fasLegs
  runLegs (map caseName table <> fasNames <> names) (map holds table <> fasProbes <> probes)
 where
  table = judgments <> refusals

names :: [String]
names =
  [ "the same request answers the same bytes twice"
  , "the clusters are never less modular than every file alone (200 seeded graphs)"
  , "the impact equals the fixpoint closure with its round depths (200 graphs, three focus sets each)"
  , "an over-cap request degrades with six empty tables and its counts; both caps at their boundary"
  , "an empty request answers six empty tables and is not degraded"
  , "the counts name the nine keys"
  , "a file is misplaced only when its cluster holds strictly more files in another directory (200 seeded graphs)"
  ]

probes :: [Bool]
probes = [deterministic, all modular [1 .. 200], all impactHolds [(i, j) | i <- [1 .. 200], j <- [0 .. 2]], capped, emptyRequest, counted, all strictMajority [1 .. 200]]

-- | A case holds when the core answers every table it names, or
-- refuses by its message.
holds :: Case -> Bool
holds c = case caseRefusal c of
  Just message -> refusedBy respond (caseRequest c) message
  Nothing -> fieldsOf respond (caseRequest c) (map fst (caseExpect c)) == Just [Just (toJSON rows) | (_, rows) <- caseExpect c]

-- | A seeded file graph: files over three directories under the
-- root, file edges and package edges drawn one in four, weights 1..3.
fileGraph :: Int -> (Int, [[Integer]], [[Integer]])
fileGraph i = (n, edges, pkgs)
 where
  n = 6 + fromInteger (draw (toInteger i) `mod` 7)
  edges = [[toInteger a, toInteger b, 1 + draw s `mod` 3] | ((a, b), s) <- zip (ordered n) (stream i), draw (s + 5) `mod` 4 == 0]
  pkgs = [[toInteger f, d, 1] | (f, s) <- zip [0 .. n - 1] (stream (i + 11)), d <- [1 .. 3], draw (s + d) `mod` 5 == 0]

dirOf :: Int -> Integer
dirOf f = 1 + toInteger f `mod` 3

fileRequest :: Int -> [Integer] -> Value
fileRequest i focus = setKey "focus" (toJSON focus) (tabledRequest "7.0.0" "arch.request" tables)
 where
  (n, edges, pkgs) = fileGraph i
  tables = [("files", [[toInteger f, dirOf f, 1] | f <- [0 .. n - 1]]), ("dirs", [[0, -1], [1, 0], [2, 0], [3, 0]]), ("edges", edges), ("pkgEdges", pkgs)]

deterministic :: Bool
deterministic = and [once r == once r | r <- map caseRequest judgments <> [fileRequest i [0] | i <- [1 .. 50]]]
 where
  once r = respond "7.0.0" (BL.toStrict (encode r))

modular :: Int -> Bool
modular i = modularity4m2 n rows part >= modularity4m2 n rows [0 .. n - 1]
 where
  (n, edges, _) = fileGraph i
  rows = [(fromInteger f, fromInteger g, w) | [f, g, w] <- edges]
  part = IM.elems (clusters n (fileNeighbours edges))

impactHolds :: (Int, Int) -> Bool
impactHolds (i, j) = fieldsOf respond (fileRequest i focus) ["impact"] == Just [Just (toJSON [[toInteger f, d] | (f, d) <- closure refs (map fromInteger focus)])]
 where
  (n, edges, pkgs) = fileGraph i
  focus = [toInteger f | (f, s) <- zip [0 .. n - 1] (stream (i * 3 + j)), draw s `mod` (toInteger j + 3) == 0]
  refs = [(fromInteger f, fromInteger g) | [f, g, _] <- edges] <> [(fromInteger f, g) | [f, d, _] <- pkgs, g <- [0 .. n - 1], dirOf g == d]

replyKeys :: [String]
replyKeys = ["layers", "cuts", "clusters", "misplaced", "impact", "metrics"]

countsOf :: [Int] -> Maybe Value
countsOf = Just . object . zipWith (.=) ["files", "dirs", "edges", "pkgEdges", "focus", "cuts", "clusters", "misplaced", "impact"]

-- | 131,073 file rows through the real respond, then the two caps at
-- their boundaries: files alone, and the two reference tables
-- counted together.
capped :: Bool
capped =
  fieldsOf respond big (["degraded", "reason", "counts"] <> replyKeys)
    == Just ([Just (Bool True), Just "arch_too_large", countsOf [cap + 1, 1, 0, 0, 0, 0, 0, 0, 0]] <> map (const (Just (toJSON ([] :: [Int])))) replyKeys)
    && not (overCap (req cap 0 0)) && overCap (req (cap + 1) 0 0)
    && not (overCap (req 0 half half)) && overCap (req 0 half (half + 1))
 where
  cap = fromInteger fileCap
  half = fromInteger refCap `div` 2
  big = tabledRequest "7.0.0" "arch.request" [("files", replicate (cap + 1) [0, 0, 0]), ("dirs", [[0, -1]])]
  req f e p = ArchReq Null (replicate f []) [] (replicate e []) (replicate p []) []

emptyRequest :: Bool
emptyRequest =
  fieldsOf respond (tabledRequest "7.0.0" "arch.request" []) (["degraded", "reason"] <> replyKeys)
    == Just ([Just (Bool False), Nothing] <> map (const (Just (toJSON ([] :: [Int])))) replyKeys)

-- | The two-hop impact case: four files, four directories, one file
-- edge, one package edge, one focus; no cut, three clusters, one
-- misplaced file, three files in the impact.
counted :: Bool
counted = [fieldsOf respond (caseRequest c) ["counts"] | c <- judgments, "the impact of a focus" `isPrefixOf` caseName c] == [Just [countsOf [4, 4, 1, 1, 1, 0, 3, 0, 3]]]

-- | The misplaced table read back against the clusters the same reply
-- answers: a file is listed, with its cluster's majority directory M
-- (most files, least id on a tie), exactly when M holds strictly more
-- of the cluster's files than the file's own directory does.
strictMajority :: Int -> Bool
strictMajority i = case fieldsOf respond (fileRequest i []) ["clusters", "misplaced"] of
  Just [Just c, Just m] | Success cs <- fromJSON c, Success ms <- fromJSON m -> ms == expected cs
  _ -> False
 where
  expected :: [[Integer]] -> [[Integer]]
  expected cs = [[f, top c] | [f, c] <- cs, held c (dirOf (fromInteger f)) < held c (top c)]
   where
    held c d = length [() | [g, c'] <- cs, c' == c, dirOf (fromInteger g) == d]
    top c = snd (maximum [(held c d, negate d) | d <- [1 .. 3], held c d > 0]) * (-1)
