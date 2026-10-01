-- | The `ce similar` console lines (plan v2.32 step 5), transcribed
-- from cli/src/similar/face.rs `console`: one header sentence — the
-- query, its terms, the candidates, how many are same-role, the
-- associative view's share under `--widen` and the degraded reason —
-- then one line per candidate: where, what, the evidence row in wire
-- order (N P C D S L), the role mark and the associative tag.
-- Advisory: no veto.
module CE.Similar.Lines (lines') where

import CE.Document.Read

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc _ =
  map (line 0) $
    say "head" [R (key "label" query), N (int "terms" query), c "candidates", c "role", P widened, P degraded]
      : map candidate (arr "candidates" doc)
 where
  query = key "query" doc
  c k = N (int k (key "counts" doc))
  widened = if bool "widen" query then say "widened" [c "widened"] else plain ""
  degraded = case key "degraded" doc of
    Null -> plain ""
    why -> say "degraded" [R why]
  candidate x =
    say
      "candidate"
      [ R (key "at" x)
      , R (key "key" x)
      , W (unwords (zipWith (\l n -> l <> show (n :: Integer)) ["N", "P", "C", "D", "S", "L"] (hits x)))
      , case key "role" x of
          Bool True -> P (say "same" [])
          Bool False -> W "-"
          _ -> W "?"
      , P (if bool "widened" x then say "associative" [] else plain "")
      ]
  hits x = case fromJSON (key "hits" x) of
    Success hs -> hs
    _ -> []
