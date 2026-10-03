-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce dedup` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/dedup/report.rs `Report`,
-- cli/src/dedup/mod.rs `Summary`): the verified clone blocks, the
-- k-way groups they aggregate into, and the twelve-number summary —
-- the walk's counts, the blocks and groups counted here, the winnowing
-- operating point (CE.Dedup.Cost) and the two report filters the run
-- used. No judgment rides the report; `--check` judges the block count
-- against the declared budget over verdict/1, and its fail bit comes
-- back here (with `check` and `budget`) for the console's ratchet line
-- and the exit.
module CE.Dedup.Document (doc) where

import CE.Dedup.Cost (dedupKgram, dedupWindow)
import CE.Document.Contract
import Data.Aeson (Value, object, (.=))
import Data.Foldable (asum)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = (docFamily "dedup" schemaId statement checked assemble ["kgram" .= dedupKgram, "window" .= dedupWindow]) {dfPretty = True}

-- | A block row is [a file, a start, a end, b file, b start, b end,
-- tokens, distinct]; a group row [g, blocks, tokens], one per group in
-- order; a member row [g, file, start, end], grouped in group order.
statement :: String
statement =
  "range paths\nrange groups\n"
    <> judgedFacts (words "files refreshed removed hot_chained stale_skipped low_diversity_suppressed min_tokens min_distinct check budget fail")
    <> "rows blocks 8 judged paths - - paths - - - -\nrows groups 3 judged groups - -\nrows members 4 judged groups paths - -\nref path paths\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ dense req "groups" (range req "groups")
    , if ordered (map (take 1) (rows req "members")) then Nothing else Just "members: not in group order"
    , bits req ["check", "fail"]
    , if flag req "fail" && not (flag req "check") then Just "facts: fail without check" else Nothing
    ]
 where
  ordered xs = and (zipWith (<=) xs (drop 1 xs))

-- | The summary's twelve counters, in the report's order.
summaryKeys :: [String]
summaryKeys = words "files refreshed removed blocks groups hot_chained stale_skipped low_diversity_suppressed kgram window min_tokens min_distinct"

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "blocks" .= map block (rows req "blocks")
    , "groups" .= map group (rows req "groups")
    , "summary" .= counted summaryKeys (map value summaryKeys)
    ]
 where
  value k = case k of
    "blocks" -> toInteger (length (rows req "blocks"))
    "groups" -> range req "groups"
    "kgram" -> dedupKgram
    "window" -> dedupWindow
    _ -> fact req k
  members = M.fromListWith (flip (<>)) [(g, [m]) | m@(g : _) <- rows req "members"]
  block r = case r of
    [af, as', ae, bf, bs, be, tokens, distinct] ->
      object
        [ "a_file" .= ref "path" [af]
        , "a_start" .= as'
        , "a_end" .= ae
        , "b_file" .= ref "path" [bf]
        , "b_start" .= bs
        , "b_end" .= be
        , "tokens" .= tokens
        , "distinct" .= distinct
        ]
    _ -> object []
  group r = case r of
    [g, blocks, tokens] -> object ["members" .= map member (M.findWithDefault [] g members), "blocks" .= blocks, "tokens" .= tokens]
    _ -> object []
  member r = case r of
    [_, f, s, e] -> object ["file" .= ref "path" [f], "start" .= s, "end" .= e]
    _ -> object []

-- | The document's schema id (the facts registry reads it here).
schemaId :: String
schemaId = "ce.dedup-report/0.5.0"
