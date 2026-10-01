-- | The `ce merge` console lines (plan v2.32 step 5), transcribed from
-- cli/src/merge/console.rs: one line of counts and of the groups not
-- sent, then per group (all, or the one `--group` names: the `only`
-- fact, its number plus one) its head line, its members and one line
-- per parameter with every member's text at it, cut to `textCap`
-- characters — the cut is the measuring side's to make on the text it
-- holds (a `clipped` reference carries the cap). No veto: a degraded
-- document is the face's own refusal, exit 2.
module CE.Merge.Lines (lines', textCap) where

import CE.Document.Read

-- | A parameter's text on one line: this many characters at most.
textCap :: Integer
textCap = 40

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req = case key "degraded" doc of
  Null -> map (line 0) (counts : concatMap group shown)
  d -> [line 0 (say "degraded" [R d])]
 where
  c k = N (int k (key "counts" doc))
  u k = N (int k (key "unsendable" doc))
  counts = say "counts" ([c "groups", c "feasible", c "holes", c "merged_duplicates"] <> map u (words "not_isomorphic no_slot_table unbuilt over_cap"))
  only = fact req "only"
  shown = [g | g <- arr "groups" doc, only == 0 || int "group" g == only - 1]
  group g =
    let family = if bool "fragment" g then say "fragment" [W (txt "family" g)] else plain (txt "family" g)
        verdict = if bool "feasible" g then say "feasible" [] else say "infeasible" [W (txt "reason" g)]
     in say "group" [N (int "group" g), P family, N (toInteger (length (arr "members" g))), N (int "params" g), N (int "savings" g), P verdict]
          : map member (arr "members" g)
          <> map param (arr "holes" g)
  member m =
    let name = if key "unit" m == Null then R (key "path" m) else R (key "unit" m)
        (a, b) = pair "lines" m
        (ra, rb) = pair "run" m
     in if (a, b) == (ra, rb) then say "member" [name, N a, N b] else say "member_run" [name, N a, N b, N ra, N rb]
  param p = say "param" [N (int "param" p + 1), P (join " | " [say "cell" [N (int "member" v), R (clipped (key "text" v))] | v <- arr "values" p])]
  pair k v = case fromJSON (key k v) of
    Success [x, y] -> (x, y)
    _ -> (0, 0 :: Integer)

-- | A text reference cut to the cap: `{"$": ["clipped", k, post,
-- postEnd, cap]}`.
clipped :: Value -> Value
clipped v = case fromJSON (key "$" v) :: Result [Value] of
  Success (_ : ints) -> case mapM fromJSON' ints of
    Just xs -> ref "clipped" (xs <> [textCap])
    Nothing -> v
  _ -> v
 where
  fromJSON' x = case fromJSON x of
    Success n -> Just n
    _ -> Nothing
