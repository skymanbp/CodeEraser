-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The Stop audit's and the git-hook faces' sentences as a document
-- family (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §6): no report document — its
-- `document` is `{}` — only what each face says and the one bit it
-- acts on. The Stop face (0) speaks only to block, its reasons joined
-- into the hook's `reason` (a gated submodule's prefixed with its
-- mount); the git-hook faces (1 precommit, 2 commitmsg) print the
-- tombstone summary, the tombstone reason when it blocks, then the
-- duplicate verdict or the staged summary, and refuse the commit on
-- either deny-tier verdict. The decisions the faces made from counts
-- are made here: the tombstone leg blocks under its deny tier, on the
-- core's `over`, over a whole measurement; the duplicate verdict
-- blocks under the guard's deny mode on its fail bit.
module CE.Audit.Document (doc) where

import CE.Document.Contract
import CE.Text
import qualified CE.Text.Audit as T
import CE.Tombstone (kindNames)
import Data.Aeson (object)
import Data.Foldable (asum)

doc :: DocFamily
doc = spoken lines' blocked (docFamily "audit" "" statement checked (const (object [])) [])

-- | `face` 0 stop / 1 precommit / 2 commitmsg; `git` 1 when the change
-- was gathered; `unreadable` 1 when commitmsg could not read its
-- message (the `message` reference); `mounted` 1 for a gated
-- submodule's audit (the `mount` reference); `mode` the guard's tier
-- (0 observe, 1 warn, 2 ask, 3 deny); `changed` the staged files. `net`
-- the net lines (one row; it may be negative); `dups` the duplicate
-- verdict [blocks, fail] (no row: it degraded) with the shown `blocks`
-- [k, a start, a end, b start, b end, tokens]; `tomb` the tombstone leg
-- [state (1 judged, 2 degraded: `error` 0), sites, label, prose,
-- erased, budget, tier, over, unread pairs, bounded pairs] (no row: no
-- leg) with the shown `places` [i, line, kind].
statement :: String
statement =
  "range blocks\nrange places\nrange errors\n\
  \fact face judged\nfact git judged\nfact unreadable judged\nfact mounted judged\n\
  \fact mode judged\nfact changed judged\n\
  \rows net 1 judged -\nrows dups 2 judged - -\nrows blocks 6 judged blocks - - - - -\n\
  \rows tomb 10 judged - - - - - - - - - -\nrows places 3 judged places - -\n\
  \ref block_a blocks\nref block_b blocks\nref place_file places\nref error errors\nref mount\nref message\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ asum [single req t | t <- ["net", "dups", "tomb"]]
    , if fact req "face" > 2 then Just "facts: face is not 0..2" else Nothing
    , asum [Just ("facts: " <> k <> " is not 0 or 1") | k <- ["git", "unreadable", "mounted"], fact req k > 1]
    , if fact req "mode" > 3 then Just "facts: mode is not 0..3" else Nothing
    , codes req "dups" 1 0 1
    , codes req "tomb" 0 1 2
    , codes req "tomb" 6 0 3
    , codes req "tomb" 7 0 1
    , codes req "places" 2 0 (toInteger (length kindNames) - 1)
    , case rows req "tomb" of
        [2 : _] | range req "errors" < 1 -> Just "tomb: degraded without an error"
        _ -> Nothing
    ]

-- | The tombstone leg's row and the duplicate verdict's, when present.
tomb, dups :: DocReq -> Maybe [Integer]
tomb req = case rows req "tomb" of [t] -> Just t; _ -> Nothing
dups req = case rows req "dups" of [d] -> Just d; _ -> Nothing

-- | The leg blocks: judged, deny tier, the core's `over`, a whole
-- measurement.
tombBlocks :: DocReq -> Bool
tombBlocks req = case tomb req of
  Just [1, _, _, _, _, _, 3, 1, unread, bounded] -> unread + bounded == 0
  _ -> False

dupBlocks :: DocReq -> Bool
dupBlocks req = fact req "mode" == 3 && fmap (take 1 . drop 1) (dups req) == Just [1]

-- | The Stop blocks / the commit is refused.
blocked :: DocReq -> Bool
blocked req = fact req "git" == 1 && fact req "unreadable" == 0 && (tombBlocks req || dupBlocks req)

lines' :: Lang -> DocReq -> [Line]
lines' lang req
  | fact req "unreadable" == 1 = [line 1 (say "unreadable" [R (ref "message" [])])]
  | fact req "git" /= 1 = [line 1 (say "not_git" [faceWord]) | face /= 0]
  | face == 0 = case reasons of
      first : rest | flag req "mounted" -> map (line 0) (say "mount" [R (ref "mount" []), P first] : rest)
      rs -> map (line 0) rs
  | otherwise = map (line 0) (summary <> [tombReason | tombBlocks req] <> [duplicate])
 where
  say = phrase T.catalogue lang
  face = fact req "face"
  faceWord = W (if face == 2 then "commitmsg" else "precommit")
  net = case rows req "net" of [[n]] -> n; _ -> 0
  reasons = [dupReason | dupBlocks req] <> [tombReason | tombBlocks req]
  dupReason = say "dup_reason" [N (maybe 0 (sum . take 1) (dups req)), W (signed net), P (join "; " (map block (rows req "blocks")))]
  block r = case r of
    [k, sa, ae, bs, be, t] -> say "block" [R (ref "block_a" [k]), N sa, N ae, R (ref "block_b" [k]), N bs, N be, N t]
    _ -> plain ""
  shown = join "; " [say "place" [R (ref "place_file" [i]), N l, W (concat (take 1 (drop (fromInteger k) kindNames)))] | [i, l, k] <- rows req "places"]
  subject = say (case face of 1 -> "subject_precommit"; 2 -> "subject_commitmsg"; _ -> "subject_stop") []
  tombReason = case tomb req of
    Just (_ : sites : _ : _ : _ : budget : _) -> say "tomb_reason" [P subject, N sites, N budget, P shown]
    _ -> plain ""
  summary = case tomb req of
    Just (2 : _) -> [say "tomb_degraded" [faceWord, R (ref "error" [0])]]
    Just [1, sites, label, prose, erased, _, tier, _, unread, bounded]
      | sites > 0 ->
          [ join "" $
              say "sites" [faceWord, N sites, N label, N prose, N erased, P shown, W (tierWord tier)]
                : [say "incomplete" [P (say "unread" [N unread, N bounded])] | unread + bounded > 0]
          ]
    _ -> []
  duplicate = case dups req of
    Nothing -> say "staged_degraded" [faceWord, N (fact req "changed"), W (signed net)]
    Just (0 : _) -> say "staged" [faceWord, N (fact req "changed"), W (signed net)]
    Just _ -> dupReason
  tierWord t = concat (take 1 (drop (fromInteger t) (words "observe warn ask deny")))
