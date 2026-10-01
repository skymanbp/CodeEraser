-- | The impact of changing the focus files (step-8 brief §1.0 ruling
-- 7; booklet §7.2): who references them, who references those, and so
-- on — a breadth-first walk over the references read backwards, each
-- file at the depth it is first met, the focus itself at 0. A package
-- reference F → D reads as F referencing every file whose directory
-- is D, since a change to any of them may reach F through D.
module CE.Arch.Impact (impact) where

import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS

-- | `[[F, depth]]` ascending by F; an empty focus answers nothing.
impact :: IM.IntMap Int -> [[Integer]] -> [[Integer]] -> [Integer] -> [[Integer]]
impact dirOf edges pkgs focus = [[toInteger f, d] | (f, d) <- IM.toAscList (walk 0 start IM.empty)]
 where
  start = IS.fromList (map fromInteger focus)
  byDir = IM.fromListWith (<>) [(d, [f]) | (f, d) <- IM.toList dirOf]
  referrers =
    IM.fromListWith
      (<>)
      ( [(fromInteger g, [fromInteger f]) | [f, g, _] <- edges]
          <> [(g, [fromInteger f]) | [f, d, _] <- pkgs, g <- IM.findWithDefault [] (fromInteger d) byDir]
      )
  walk depth frontier seen
    | IS.null frontier = seen
    | otherwise = walk (depth + 1) next seen'
   where
    seen' = IM.union seen (IM.fromSet (const depth) frontier)
    next = IS.fromList [r | f <- IS.toList frontier, r <- IM.findWithDefault [] f referrers, not (IM.member r seen')]
