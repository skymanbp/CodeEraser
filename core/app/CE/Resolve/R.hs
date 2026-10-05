-- | R rungs (plan v2.30 step 4; design booklet §8 row R; moved from
-- cli/src/graph/ladder/r/mod.rs in plan v2.33 W2-text stage B — each
-- function below is the Rust function of the same name). R has no
-- import statement: `source("x.R")` runs a file, and `library(pkg)`,
-- `requireNamespace("pkg")` and `pkg::name` reach a package (the
-- measuring side's detector labels the last three `library`). The sites
-- walk:
--   R1 `source`: the path beside the sourcing file, then under the tree
--      root — a path is relative to R's working directory, the project
--      root by the convention RStudio projects and `Rscript` runs from
--      the root share — the first hit; then the declared
--      `[graph.search_roots] r` directories, two of which answering two
--      different files is ambiguous_root;
--   R2 `library`: the directory of the carried DESCRIPTION whose
--      `Package:` field names the package (CE.Resolve.Description) — a
--      package is its directory, its code the `R/` files under it (the
--      reply's `packages`); two such is ambiguous_workspace;
--   R3 External: a package no carried DESCRIPTION declares — base R,
--      CRAN and Bioconductor live outside the corpus by construction, so
--      no name table is needed — and a `source` of a URL.
-- A `source` no rung holds is out_of_scope: a file outside the tree, or a
-- path spelled for a working directory the text does not name.
module CE.Resolve.R (resolveR) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Description (Description (..))
import CE.Resolve.Tables (kindLibrary, kindSource)
import CE.Resolve.World (World, besideOrRoot, declaredIn)
import Data.List (isInfixOf)

-- | `resolve`: one site by its kind, over the tree, the declared `r`
-- roots and the DESCRIPTIONs that read.
resolveR :: (World, [String], [Description]) -> Integer -> String -> String -> Answer
resolveR (w, roots, descs) kind from spec
  | kind == kindSource && "://" `isInfixOf` spec = AExternal 3
  | kind == kindSource = firstOf [(`AFile` 1) <$> besideOrRoot w from spec, oneOf (declaredIn w roots spec) 1] (AUnresolved OutOfScope)
  | kind == kindLibrary = package descs spec
  | otherwise = AUnresolved Unsupported

-- | `package`: R2, the one carried package of that name, or External when
-- the tree declares none.
package :: [Description] -> String -> Answer
package descs spec = case [d | d <- descs, dPackage d == spec] of
  [] -> AExternal 3
  [one] -> APackage (dDir one) 2
  _ -> AUnresolved AmbiguousWorkspace
