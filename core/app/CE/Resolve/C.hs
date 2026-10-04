-- | C / C++ rungs (plan v2.30 step 2; the compile database completed in
-- step 5b, item 14; the `include` rung in step 6; design booklet §8 row
-- C / C++; moved from cli/src/graph/ladder/c.rs in plan v2.33 wave W2a,
-- on text since W2-text — each function below is the Rust function of
-- the same name). The site is a `preproc_include`; its form (`"x.h"` or
-- `<x.h>`) IS the search order the language defines (C17 §6.10.2):
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
import CE.Resolve.Flags (Chain (..))
import CE.Resolve.Str
import CE.Resolve.World
import qualified Data.Set as Set

-- | `resolve`: one site, its file and specifier.
resolveC :: Env -> Index -> String -> String -> Answer
resolveC env ix from spec
  | null name = AUnresolved Empty
  | otherwise =
      firstOf
        [ if own then beside else Nothing
        , oneOf (declared env name) 2
        , oneOf (Set.unions [along w (ch c) system name (stack (ch c) (ixParents ix !! c) from) | c <- Set.toList chains]) 3
        , if Set.null chains then includeRung else Nothing
        ]
        (if system then AExternal 5 else AUnresolved OutOfScope)
 where
  (name, system) = form spec
  w = eWorld env
  ch c = ixChains ix !! c
  chains = chainsOf ix from
  own = not system && (Set.null chains || any (chOwnDir . ch) (Set.toList chains))
  beside = case joinRel (parentDir from) name of
    Just p | member w p -> Just (AFile p 1)
    _ -> Nothing
  includeRung = oneOf (Set.fromList [p | d <- ancestors (parentDir from), Just p <- [inScope w (joinDir d "include") name]]) 4
