-- | C / C++ rungs (plan v2.30 step 2; the compile database completed in
-- step 5b, item 14; the `include` rung in step 6; design booklet
-- §8 row C / C++; moved from cli/src/graph/ladder/c.rs in plan v2.33
-- wave W2a). The site is a `preproc_include`; its form (`"x.h"` or `<x.h>`)
-- IS the search order the language defines (C17 §6.10.2):
--   R1 the including file's own directory, for the quoted form only —
--      unless every compile of the file passes `-I-`;
--   R2 the declared roots, `[graph.search_roots] c` (C++ shares the
--      key) — two directories holding two different files is
--      ambiguous_root: the declaration named no order;
--   R3 the compile database (CE.Resolve.CIndex): the chains the file
--      compiles under, each searched in the preprocessor's class order,
--      first hit per chain; two chains answering two files is
--      ambiguous_root — a name the build compiles as two files;
--   R4 the `include` directory beside the including file's own
--      directory or beside any ancestor of it, the tree root among them
--      — both forms, and only for a file no compile chain reaches;
--   R5 External: the system form with no hit is a toolchain or system
--      header. The quoted form with no hit is out_of_scope.
-- NEVER a basename search of the tree: two `util.h` in one repository
-- are two files, and picking one would invent an edge.
module CE.Resolve.C (resolveC) where

import CE.Resolve.Answer
import CE.Resolve.CIndex
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Vocab (Word' (..))
import CE.Resolve.World
import Data.Array ((!))
import qualified Data.IntSet as IS

-- | One site: the including file, the system form, the name's pieces.
resolveC :: World -> Index -> Int -> Bool -> [Int] -> Answer
resolveC w ix from system pieces
  | pieces == [word w WEmpty] = AUnresolved Empty
  | otherwise =
      firstOf
        [ if own then (`AFile` 1) <$> inScope w (parentOf w from) name else Nothing
        , oneOf (hits [inScope w r name | r <- ixRoots ix]) 2
        , oneOf (IS.toList (IS.fromList (concatMap database chains))) 3
        , if null chains then oneOf (hits [inScope w (d <> [word w WInclude]) name | d <- ancestors (parentOf w from)]) 4 else Nothing
        ]
        (if system then AExternal 5 else AUnresolved OutOfScope)
 where
  name = map K pieces
  chains = chainsOf ix from
  own = not system && (null chains || any (chOwn . (ixChains ix !)) chains)
  database c = along w (ixChains ix ! c) system name (stackOf w (ixChains ix ! c) (ixParents ix ! c) from)
