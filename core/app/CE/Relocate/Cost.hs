-- | The relocation stage's constants (v2.35 guard lane, GD4): the row
-- cap and the one rule. Rust counts, for each match a would-refuse
-- write has, the occurrences of the matched token run in the tree at
-- HEAD and in the tree the write leaves (the changed and untracked
-- files of every fingerprinting language, the target as applied);
-- whether that is a relocation is judgment and lives here.
module CE.Relocate.Cost (relocateRowCap, relocated) where

-- | Row ceiling: one row per match a write's probe left after the
-- novelty subtraction. A write matching thousands of regions is not a
-- move; over the cap the core answers a complete degraded reply and
-- the hook refuses as it did before the stage existed.
relocateRowCap :: Integer
relocateRowCap = 4096

-- | One match is a relocation when the write does not raise the tree's
-- count of its run: `occNow <= occHead`. Unchanged files count alike
-- on both sides, so the two counts are the tree-wide ones.
relocated :: [Integer] -> Bool
relocated [occHead, occNow] = occNow <= occHead
relocated _ = False -- unreachable behind the row contract; never credit
