-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The PreToolUse guard's sentences as a document family (plan v2.32
-- step 5; design booklet docs/reference/authority-track.md §6): the
-- guard has no report document — its `document` is `{}` — only the
-- sentences it writes as a decision's reason, one line per rule that
-- spoke, in the order the hook gathered them (the hook joins them and
-- picks the decision tier; the tier is the hook's). Every rule's
-- integers ride one `say` row, its strings as references: the path the
-- write names, the probe's matched files, the tombstone sites' files,
-- the flow finding's unit, the config error.
module CE.Guard.Document (doc) where

import CE.Document.Contract
import CE.Document.Read (Say, spoken)
import qualified CE.Flow.Document as Flow
import CE.Text
import qualified CE.Text.Guard as T
import CE.Tombstone (kindNames)
import Data.Aeson (Value, object)
import Data.Foldable (asum)

doc :: DocFamily
doc = spoken T.catalogue lines' (\_ _ -> False) (docFamily "guard" "" statement checked (const (object [])) [])

-- | A `say` row is [rule, a, b, c, d, e, f]: rule 0 duplicate [file,
-- regions, first match, matches shown], 1 over the hard budget [file,
-- lines, cap, fence], 2 the graded zone [file, lines, permille, soft,
-- cap], 3 tombstone [sites, budget, fence, first place, places shown],
-- 4 a new flow finding [file, novel, unit, kind, line], 5 an
-- unreadable ce.toml [error]; unused columns 0. A fence is 0 none, 1
-- the config drifted from its baseline, 2 the baseline unreadable. A
-- match row is [k, start, end, tokens], a place row [i, line, kind].
statement :: String
statement =
  "range files\nrange matches\nrange places\nrange units\nrange errors\n\
  \rows say 7 judged - - - - - - -\n\
  \rows matches 4 judged matches - - -\nrows places 3 judged places - -\n\
  \ref file files\nref match_file matches\nref place_file places\nref unit units\nref error errors\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ codes req "say" 0 0 5
    , codes req "places" 2 0 (toInteger (length kindNames) - 1)
    , asum (zipWith row [0 :: Int ..] (rows req "say"))
    ]
 where
  row i r = case r of
    [0, f, _, a, n, _, _] -> asum [inside "files" f, span' "matches" a n]
    [1, f, _, _, fence, _, _] -> asum [inside "files" f, fenced fence]
    [2, f, _, _, _, _, _] -> inside "files" f
    [3, _, _, fence, a, n, _] -> asum [fenced fence, span' "places" a n]
    [4, f, _, u, k, _, _] -> asum [inside "files" f, inside "units" u, if k < 0 || k >= toInteger (length Flow.kindTable) then Just "kind" else Nothing]
    [5, e, _, _, _, _, _] -> inside "errors" e
    _ -> Nothing
   where
    named what = fmap (\w -> "say " <> show i <> ": " <> what <> " " <> w)
    inside u x = named "out of range" (if x < 0 || x >= range req u then Just u else Nothing)
    span' u a n = named "out of range" (if a < 0 || n < 0 || a + n > range req u then Just u else Nothing)
    fenced fence = named "not a fence" (if fence < 0 || fence > 2 then Just (show fence) else Nothing)

lines' :: Say -> Value -> DocReq -> [Line]
lines' say _ req = map (line 0 . sentence) (rows req "say")
 where
  file f = R (ref "file" [f])
  slice t a n = take (fromInteger n) (drop (fromInteger a) (rows req t))
  note fence = case fence of
    1 -> say "note" [P (say "drifted" [])]
    2 -> say "note" [P (say "baseline_unreadable" [])]
    _ -> plain ""
  sentence r = case r of
    [0, f, regions, a, n, _, _] ->
      say "duplicate" [file f, N regions, P (join "; " [say "match" [R (ref "match_file" [k]), N s, N e, N t] | [k, s, e, t] <- slice "matches" a n])]
    [1, f, ls, cap, fence, _, _] -> say "over_budget" [file f, N ls, N cap, P (note fence)]
    [2, f, ls, permille, soft, cap, _] -> say "graded_zone" [file f, N ls, N permille, N soft, N cap]
    [3, sites, budget, fence, a, n, _] ->
      let shown = join "; " [say "place" [R (ref "place_file" [i]), N l, W (kindName k)] | [i, l, k] <- slice "places" a n]
       in join "" [say "tombstone_over" [N sites, N budget, P shown], note fence]
    [4, f, novel, u, k, l, _] -> say "flow_novel" [N novel, file f, R (ref "unit" [u]), W (flowKind k), N l]
    [5, e, _, _, _, _, _] -> say "config_unreadable" [R (ref "error" [e])]
    _ -> plain ""
  kindName k = concat (take 1 (drop (fromInteger k) kindNames))
  flowKind k = concat (take 1 (drop (fromInteger k) Flow.kinds))
