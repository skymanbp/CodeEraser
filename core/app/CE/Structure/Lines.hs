-- | The `ce structure` console lines (plan v2.32 step 5), transcribed
-- from cli/src/structure/report.rs: the score line, the layout
-- divergence when directories were declared, the deviations, the
-- findings, and under `--split-candidates` the advisory — a
-- candidate's ROI is benefit over cost to one decimal (CE.Text.fixed:
-- the double the measuring side divided, rounded as it rounds), an
-- exemption's reason by whether any seam existed. Report-only: no veto.
module CE.Structure.Lines (lines') where

import CE.Document.Contract
import CE.Text
import qualified CE.Text.Structure as T

lines' :: Lang -> DocReq -> [Line]
lines' lang req =
  map (line 0) $
    [say "head" [N (fact req "score"), N (fact req "scale"), W (pairs "entropy"), W (pairs "axes"), N (range req "dirs")]]
      <> divergence
      <> [say "deviation" [R (ref "dir" [d]), P (say (if k == 0 then "undeclared" else "declared_empty") [])] | [d, k] <- rows req "deviations"]
      <> [say "finding" [R (ref "dir" [d]), N a] | [d, a] <- rows req "findings"]
      <> (if flag req "split" then candidates <> exempt else [])
 where
  say = phrase T.catalogue lang
  pairs t = unwords [show k <> ":" <> show v | [k, v] <- rows req t]
  declared = fact req "declaredDirs"
  divergence
    | declared <= 0 = []
    | [d] : _ <- rows req "divergence" = [say "divergence" [N d, N declared]]
    | otherwise = [say "divergence_undefined" [N declared]]
  candidates =
    [ say "split" [R (ref "path" [f]), N after, R (ref "unit" [f, u]), P (say "roi" [W (fixed 1 b c)]), N b, N c]
    | [f, u, b, c, after] <- rows req "splitCandidates"
    ]
  exempt =
    [ say "exempt" [R (ref "path" [f]), P (say (if c == 0 then "no_seam" else "cohesive") []), N b, N c]
    | [f, b, c] <- rows req "sizeExempt"
    ]
