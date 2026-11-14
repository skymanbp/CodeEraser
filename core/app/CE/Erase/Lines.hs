-- | The `ce erase` console lines (plan v2.32 step 5), transcribed from
-- cli/src/erase/render.rs `print` and cli/src/main_erase.rs: the
-- unified diff, one reference per file (the eraseable rows of one path,
-- in plan order — the measuring side renders the hunks and checks the
-- hash), each advisory row with its reason (and the unresolved sites
-- behind `language_unresolved`), each out-of-class kind with its family
-- command, the plan summary (a dry run's or an applying run's, by the
-- `apply` flag; 1.9.2 W2); under `--check` (the `check` fact) the
-- refusal on stderr when a row is eraseable; after `--apply` (the
-- `apply` flag) the apply line with the `applied` count. The veto is
-- that refusal.
module CE.Erase.Lines (lines', spanText, veto) where

import CE.Document.Read
import qualified CE.Erase.Document as Erase
import Data.Function (on)
import Data.List (groupBy)

lines' :: Say -> Value -> DocReq -> [Line]
lines' say doc req =
  map (line 0) (diffs <> advisories <> map kind kinds <> [say summary [N (c "eraseable"), N (c "advisory")]])
    <> [line 1 (say "check" [N (c "eraseable")]) | veto doc req]
    <> [line 0 (say "applied" [N (fact req "applied")]) | flag req "apply"]
 where
  c k = int k (key "counts" doc)
  summary = if flag req "apply" then "summary_applied" else "summary_dry"
  plan = zip (Erase.planRows req) (arr "rows" doc)
  eraseable = [(i, r) | ((i, r), _) <- plan, take 1 (drop 7 r) == [1]]
  diffs = [say "diff" [R (ref "diff" [a, foldl (\_ (i, _) -> i) a g])] | g@((a, _) : _) <- groupBy ((==) `on` (take 1 . drop 1 . snd)) eraseable]
  advisories = [advisory r | (_, r) <- plan, not (bool "eraseable" r)]
  advisory r =
    say
      "advisory"
      [ W (txt "class" r)
      , R (key "path" r)
      , W (spanText (key "span" r))
      , W (txt "reason" r)
      , P (if txt "reason" r == "language_unresolved" then say "detail" [N (int "sites" r)] else plain "")
      , P (provenance say (key "from" r))
      ]
  kinds = [(k, n) | (k, Number n) <- fields (key "out_of_class" (key "counts" doc))]
  kind (k, n) = say "kind" [W k, N (round n), fillOf (key k (key "families" doc))]

-- | A row's provenance clause in the console's language, written from
-- its facts (release 1.10.0; the document's `provenance` keeps the
-- English sentence the measuring side wrote, which the English clause
-- equals): the dead file's verdict word (a document word in both
-- languages) and its reason, a doc twin's survivor, Jaccard and
-- verbatim count and the links its section receives, a clone twin's
-- other copy, tokens and covered unit.
provenance :: Say -> Value -> Piece
provenance say f = case txt "family" f of
  "deadcode" -> say "prov_dead" [W (txt "verdict" f), P (say (whyKey (int "whyCode" f)) [])]
  "docdup" ->
    let linked = int "linked" f
        tail' = if linked > 0 then say "prov_linked" [N linked] else plain ""
     in say "prov_doc" (twin <> [N (nth "jaccard" 0), N (nth "jaccard" 1), N (int "verbatim" f), P tail'])
  _ -> say "prov_twin" (twin <> [N (int "tokens" f), R (key "unit" f)])
 where
  whyKey c = if c == 1 then "why_unreach" else "why_unref"
  twin = [R (key "twin" f), N (nth "span" 0), N (nth "span" 1)]
  nth k n = case fromJSON (key k f) of
    Success xs | length (xs :: [Integer]) > n -> xs !! n
    _ -> 0

-- | `:start-end`, or nothing for a whole-file row (the trail's
-- records read it too).
spanText :: Value -> String
spanText v = case fromJSON v of
  Success [s, e] -> ":" <> show (s :: Integer) <> "-" <> show e
  _ -> ""

-- | `--check` with an eraseable row planned.
veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && int "eraseable" (key "counts" doc) > 0
