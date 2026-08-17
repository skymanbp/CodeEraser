-- | The A-layer declared template (M6 S3, design booklet §2 row A):
-- ce.toml's [structure] layout arrives as validated [dirId, weight]
-- rows and every file's owner is its DEEPEST declared ancestor (the
-- R1 cabal-root precedent — nesting resolves by depth, never by
-- guess). Observed mass on territory no declaration owns makes the
-- χ² divergence undefined; those directories are reported BY NAME
-- (kind 0) instead of collapsing into a number, and declared bins
-- that own nothing are named too (kind 1) — an expectation with no
-- reality is a deviation, not a zero to average away.
module CE.Structure.Declared (declaredRows) where

import CE.Structure.Entropy (chi2, perMille)
import Data.List (sort)
import qualified Data.Map.Strict as M

-- | Nothing = no declaration (the reply carries no A-layer keys at
-- all — the S2 shape stays byte-identical). Just (divergence row,
-- deviation rows) otherwise: the divergence row is [perMille χ²]
-- when the declared bins own every file, [] when undeclared
-- territory holds mass (the deviation rows then say where).
declaredRows :: [[Integer]] -> [[Integer]] -> Maybe ([Integer], [[Integer]])
declaredRows nodes declared
  | null declared = Nothing
  | otherwise = Just (divergenceRow, sort (unownedRows <> emptyRows))
 where
  owner = ownersOf nodes (M.fromList [(d, ()) | [d, _] <- declared])
  ownedBy =
    M.fromListWith
      (+)
      [(o, files) | [i, _, _, _, files] <- nodes, Just o <- [M.lookup i owner]]
  unowned =
    [i | [i, _, _, _, files] <- nodes, files > 0, M.notMember i owner]
  unownedRows = [[i, 0] | i <- unowned]
  emptyRows =
    [[d, 1] | [d, _] <- declared, M.findWithDefault 0 d ownedBy == 0]
  divergenceRow
    | not (null unowned) = []
    | otherwise =
        -- weights are validated >= 1, so the zero-reference-bin
        -- Nothing arm of chi2 is unreachable here; maybe keeps the
        -- call total without a partial extraction
        maybe [] (pure . perMille) (chi2 bins)
  bins = [(M.findWithDefault 0 d ownedBy, w) | [d, w] <- declared]

-- | Deepest-declared-ancestor map, built in one pass: ids are dense
-- and parents precede children (the boundary contract), so a
-- directory's owner is itself when declared, else its parent's
-- owner — the root has no fallback (absent = unowned).
ownersOf :: [[Integer]] -> M.Map Integer () -> M.Map Integer Integer
ownersOf nodes declSet = foldl step M.empty nodes
 where
  step m row = case row of
    [i, p, _, _, _]
      | M.member i declSet -> M.insert i i m
      | i /= p, Just o <- M.lookup p m -> M.insert i o m
    _ -> m
