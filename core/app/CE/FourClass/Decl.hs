-- | Declaration-level relocation — the judgment (7.1.0, O48). A unit
-- whose body cannot clear the line-evidence floor still has an
-- identity: name, kind and arity. When that identity vanishes from
-- one pair and appears in EXACTLY ONE other inside a single batch,
-- the key itself is the provenance a cross site must buy, so it pays
-- siteCostCross and what remains to be paid is content — declFloor,
-- which is 1 (CE.FourClass.Cost).
--
-- One destination, many sources: the same asymmetry Anchor states for
-- lines (de-duplication commits are many-to-one, and the destination
-- is where invention would live). Two destinations refuse the key
-- outright — no tie-break, no score, nothing fuzzy anywhere here.
module CE.FourClass.Decl (declCap, declViolation, edges) where

import CE.FourClass.Cost (declFloor)
import CE.FourClass.Wire (DeclRow, Pair (..))
import Control.Applicative ((<|>))
import Data.Maybe (isJust)
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import Data.Word (Word64)

-- | Declaration rows one batch may carry, both sides summed. Over it
-- the edge table is answered EMPTY with `unitEdgesDropped` — refused,
-- never truncated (the graph family's unmentionedDropped posture).
declCap :: Int
declCap = 65536

type Key = (Word64, Int)

keyOf :: DeclRow -> Key
keyOf (h, k, _, _) = (h, k)

spanOf :: DeclRow -> (Int, Int)
spanOf (_, _, s, e) = (s, e)

remRows :: Pair -> [DeclRow]
remRows = concat . pDeclRem

addRows :: Pair -> [DeclRow]
addRows = concat . pDeclAdd

-- | Boundary offences of the 7.1.0 tables, in order: half a table,
-- then a partly measured batch (destination uniqueness is batch-wide,
-- so a partial batch cannot be judged), then a malformed span or kind
-- word, then a key sent twice on one side (the Rust producer keys on
-- multiplicity one, so this refuses drift, not traffic).
declViolation :: [Pair] -> Maybe String
declViolation ps = paired <|> uniform <|> spans <|> dups
 where
  named what is = case is of
    (i : _) -> Just (what <> ": pair " <> show i)
    [] -> Nothing
  measured p = isJust (pDeclRem p)
  paired =
    named
      "decl tables come in pairs"
      [pIdx p | p <- ps, isJust (pDeclRem p) /= isJust (pDeclAdd p)]
  uniform
    | not (any measured ps) || all measured ps = Nothing
    | otherwise = named "decl tables must cover every pair" [pIdx p | p <- ps, not (measured p)]
  spans = named "malformed decl span" [pIdx p | p <- ps, any bad (remRows p <> addRows p)]
  bad (_, k, s, e) = s < 1 || e < s || k < 1 || k > 4
  dups = named "duplicate decl key" [pIdx p | p <- ps, any repeats [remRows p, addRows p]]
  repeats rs = S.size (S.fromList (map keyOf rs)) /= length rs

-- | The accepted edges, ascending, and whether the cap tripped.
-- Nothing = no pair carried the tables: a pre-7.1.0 aligner, and the
-- family answers exactly as it did before.
edges :: [Pair] -> Maybe ([(Int, Int, Word64)], Bool)
edges ps
  | not (any (isJust . pDeclRem) ps) = Nothing
  | overCap = Just ([], True)
  | otherwise = Just (S.toAscList accepted, False)
 where
  overCap = sum [length (remRows p) + length (addRows p) | p <- ps] > declCap
  adders = M.fromListWith (<>) [(keyOf r, [(pIdx p, spanOf r)]) | p <- ps, r <- addRows p]
  vanished = [(keyOf r, (pIdx p, spanOf r)) | p <- ps, r <- remRows p]
  hashesBy side = M.fromListWith (<>) [(pIdx p, [(l, h)]) | p <- ps, (l, h, _) <- concat (side p)]
  remHashes = hashesBy pRem
  addHashes = hashesBy pAdd
  within m i (s, e) = S.fromList [h | (l, h) <- M.findWithDefault [] i m, l >= s, l <= e]
  -- the singleton pattern IS the ambiguous-destination refusal: a key
  -- two pairs claim matches nothing and contributes no edge
  accepted =
    S.fromList
      [ (from, to, fst k)
      | (k, (from, rspan)) <- vanished
      , [(to, aspan)] <- [M.findWithDefault [] k adders]
      , to /= from
      , S.size (S.intersection (within remHashes from rspan) (within addHashes to aspan))
          >= declFloor
      ]
