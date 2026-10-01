-- | The `ce erase` console lines (plan v2.32 step 5), transcribed from
-- cli/src/erase/render.rs `print` and cli/src/main_erase.rs: the
-- unified diff, one reference per file (the eraseable rows of one path,
-- in plan order — the measuring side renders the hunks and checks the
-- hash), each advisory row with its reason (and the unresolved sites
-- behind `language_unresolved`), each out-of-class kind with its family
-- command, the plan summary; under `--check` (the `check` fact) the
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
  map (line 0) (diffs <> advisories <> map kind kinds <> [say "summary" [N (c "eraseable"), N (c "advisory")]])
    <> [line 1 (say "check" [N (c "eraseable")]) | veto doc req]
    <> [line 0 (say "applied" [N (fact req "applied")]) | flag req "apply"]
 where
  c k = int k (key "counts" doc)
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
      , R (key "provenance" r)
      ]
  kinds = [(k, n) | (k, Number n) <- fields (key "out_of_class" (key "counts" doc))]
  kind (k, n) = say "kind" [W k, N (round n), fillOf (key k (key "families" doc))]

-- | `:start-end`, or nothing for a whole-file row (the trail's
-- records read it too).
spanText :: Value -> String
spanText v = case fromJSON v of
  Success [s, e] -> ":" <> show (s :: Integer) <> "-" <> show e
  _ -> ""

-- | `--check` with an eraseable row planned.
veto :: Value -> DocReq -> Bool
veto doc req = flag req "check" && int "eraseable" (key "counts" doc) > 0
