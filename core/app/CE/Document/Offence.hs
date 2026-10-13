-- | The document contract's boundary check, split out of
-- CE.Document.Contract by its own seam (plan v2.33 step 11a): the first
-- offender against a family's statement, named.
module CE.Document.Offence (offence) where

import CE.Document.Contract (DocFamily (..), DocReq (..), Spec (..), Table (..), fact, rows)
import CE.Wire (label)
import Control.Monad (guard)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import Data.Maybe (isNothing)

-- | The first offender against a family's statement, in request
-- order: the three objects present, each holding exactly its stated
-- keys, no negative range or fact, every row its width with every
-- indexing column inside its universe, the reason inside `why`, and
-- under a reason no judged row or fact; then the family's own checks.
offence :: DocFamily -> DocReq -> Maybe String
offence fam req = case (dRanges req, dRows req, dFacts req) of
  (Nothing, _, _) -> Just "document: missing ranges"
  (_, Nothing, _) -> Just "document: missing rows"
  (_, _, Nothing) -> Just "document: missing facts"
  (Just rs, Just ts, Just fs) ->
    asum
      [ keyed "ranges" (spRanges sp) [] rs
      , negative "ranges" rs
      , keyed "rows" (map tName (spTables sp)) (spOptional sp) ts
      , asum [shaped rs t (M.findWithDefault [] (tName t) ts) | t <- spTables sp]
      , keyed "facts" (map fst (spFacts sp)) (spOptional sp) fs
      , negative "facts" fs
      , reason rs
      , asum [judgedWith t | t <- spTables sp, not (tKept t), not (null (rows req (tName t)))]
      , asum [factWith n | (n, False) <- spFacts sp, fact req n /= 0]
      , dfCheck fam req
      ]
 where
  sp = dfSpec fam
  reason rs = do
    i <- dDegraded req
    guard (i < 0 || i >= M.findWithDefault 0 "why" rs)
    pure "degraded: out of range why"
  judgedWith t = do
    _ <- dDegraded req
    pure ("degraded: with rows " <> tName t)
  factWith n = do
    _ <- dDegraded req
    pure ("degraded: with fact " <> n)

-- | An object holding exactly the stated keys (an optional one may be
-- absent): the first missing one, else the first unknown one.
keyed :: String -> [String] -> [String] -> M.Map String a -> Maybe String
keyed what stated optional' m =
  asum
    [ asum [Just (what <> ": missing " <> k) | k <- stated, k `notElem` optional', isNothing (M.lookup k m)]
    , asum [Just (what <> ": unknown key " <> k) | k <- M.keys m, k `notElem` stated]
    ]

negative :: String -> M.Map String Integer -> Maybe String
negative what m = asum [Just (what <> ": " <> k <> " is negative") | (k, v) <- M.toList m, v < 0]

-- | Every row of a table its width, every indexing column inside its
-- universe: `<table> <i>: <reason>`.
shaped :: M.Map String Integer -> Table -> [[Integer]] -> Maybe String
shaped rs t = asum . zipWith row [0 :: Int ..]
 where
  row i r
    | length r < tWidth t || (not (tOpen t) && length r > tWidth t) =
        Just (label (tName t) i <> "malformed row (need " <> show (tWidth t) <> (if tOpen t then "+" else "") <> " columns)")
    | otherwise = asum (zipWith (cell i) [0 :: Int ..] (zip (tCols t) r))
  cell i c (Just u, v)
    | v < 0 || v >= M.findWithDefault 0 u rs = Just (label (tName t) i <> "column " <> show c <> " out of range " <> u)
  cell _ _ _ = Nothing
