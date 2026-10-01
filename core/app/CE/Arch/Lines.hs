-- | The `ce arch` console lines (plan v2.32 step 5), transcribed from
-- cli/src/arch/console.rs: one counts line, the layers from the top
-- level down, every cut arc with the file references it folds, the
-- misplaced files, the impact walk when a focus was named, and the
-- metrics table padded to its widest directory; an empty section says
-- so in one sentence, a degraded document is one line naming why. The
-- root directory prints as `.`, a cut's ends slashed. No veto: the
-- face exits 2 on a degraded document itself (a refusal).
module CE.Arch.Lines (lines') where

import CE.Document.Read
import Data.List (sortOn)
import qualified Data.IntMap.Strict as IM
import Data.Ord (Down (..))

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req = case dDegraded req of
  Just _ -> [line 0 (say "degraded" [R (whyRef req)])]
  Nothing -> map (line 0) (counts : layers <> cuts <> misplaced <> impact <> metrics)
 where
  c k = N (int k (key "counts" doc))
  counts = say "counts" (map c (words "files dirs edges pkgEdges cuts clusters misplaced"))
  widths = IM.fromList [(fromInteger d, (b, n)) | [d, b, n] <- rows req "widths"]
  -- `.` for the root (its path is empty), else the path
  shown d = case IM.lookup (fromInteger d) widths of
    Just (0, _) -> W "."
    _ -> R (ref "dir" [d])
  layers =
    [ say "level" [N l, P (join ", " [piece (shown d) | d <- ds])]
    | (l, ds) <- grouped (sortOn (Down . fst) [(l, d) | [d, l] <- rows req "layers"])
    ]
  cuts
    | null (rows req "cuts") = [say "no_cuts" []]
    | otherwise = concat (zipWith cut (rows req "cuts") (arr "cuts" doc))
  cut row d = case row of
    [a, b, w, e] ->
      say "cut" [R (ref "slashed" [a]), R (ref "slashed" [b]), N w, P (say (if e == 1 then "exact" else "greedy") [])]
        : [say "cut_file" [R (key "from" f), R (key "to" f), N (int "refs" f)] | f <- arr "files" d]
    _ -> []
  dirAt = IM.fromList [(fromInteger f, d) | [f, d, _] <- rows req "files"]
  misplaced
    | null (rows req "misplaced") = [say "no_misplaced" []]
    | otherwise = [say "misplaced" [R (ref "path" [f]), shown (IM.findWithDefault (-1) (fromInteger f) dirAt), shown m] | [f, m] <- rows req "misplaced"]
  impact
    | null (rows req "focus") = []
    | otherwise = say "impact_head" [] : [say "impact" [R (ref "path" [f]), N depth] | [f, depth] <- rows req "impact"]
  -- the widest shown path (bytes, as Rust measures `len`), 3 at least;
  -- each row pads by characters to it
  width = maximum (3 : [fromInteger (shownBytes d) | [d, _, _, _] <- rows req "metrics"])
  shownBytes d = maybe 0 (\(b, _) -> if b == 0 then 1 else b) (IM.lookup (fromInteger d) widths)
  shownChars d = maybe 0 (\(b, n) -> if b == 0 then 1 else n) (IM.lookup (fromInteger d) widths)
  metrics = say "metrics_head" [W (leftIn width "dir")] : map metric (rows req "metrics")
  metric row = case row of
    [d, fanIn, fanOut, s] ->
      say
        "metrics_row"
        [ shown d
        , W (replicate (width - fromInteger (shownChars d)) ' ')
        , W (rightIn 5 (show fanIn))
        , W (rightIn 6 (show fanOut))
        , W (rightIn 11 (if s >= 0 then show s else "-"))
        ]
    _ -> plain ""

-- | Consecutive runs of equal keys, in order.
grouped :: [(Integer, Integer)] -> [(Integer, [Integer])]
grouped xs = case xs of
  [] -> []
  (k, _) : _ -> let (same, rest) = span ((== k) . fst) xs in (k, map snd same) : grouped rest
