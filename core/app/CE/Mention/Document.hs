-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce graph --mentions` document (plan v2.32 step 4; design
-- booklet docs/reference/authority-track.md §5), transcribed from the
-- face that assembled it on the measuring side (cli/src/mention/face.rs
-- `report_json` over cli/src/mention/mod.rs `Stats`,
-- cli/src/mention/census.rs `Outside` and cli/src/mention/rates.rs
-- `census`): the mention pass's header, its skips, this run's
-- movement, the judged files outside the universe, and the veto's
-- census per language. No judgment rides this document — every number
-- is the measuring side's own, `mention_rev` its index revision
-- included — but its shape, its schema id and the language names are
-- stated here, once. A pass that fails prints no document (the face
-- exits 2), so the family takes no reason.
module CE.Mention.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value, object, (.=))
import Data.Aeson.Key (fromString)
import Data.Aeson.Types (Pair)
import Data.Foldable (asum)
import Data.List (nub)

doc :: DocFamily
doc = docFamily "mentions" schemaId statement checked assemble []

schemaId :: String
schemaId = "ce.mentions-report/0.3.0"

-- | The document's numbers, a dotted name one level down; `run.rescanned`
-- is the one boolean.
header :: [String]
header =
  words
    "mention_rev universe sources rows capped dist_js_dedup_runs \
    \skipped.oversize skipped.binary skipped.signed skipped.walk_errors \
    \run.refreshed run.removed run.rescanned run.clipped run.starved \
    \outside.oversize outside.binary outside.nested outside.ignored"

-- | The request: the header as facts, one census row per language
-- [language code, declared, declared exported, unmentioned,
-- unmentioned exported, vetoed by another file, by a fold key, by the
-- file's own text, saved by a collision].
statement :: String
statement = concatMap (\k -> "fact " <> k <> " judged\n") header <> "rows rates 9 judged - - - - - - - - -\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ if fact req "run.rescanned" > 1 then Just "facts: run.rescanned is not 0 or 1" else Nothing
    , asum [Just ("rates " <> show i <> ": no language " <> show c) | (i, c : _) <- zip [0 :: Int ..] (rows req "rates"), null (langName c)]
    , if length (nub codes') == length codes' then Nothing else Just "rates: a language twice"
    ]
 where
  codes' = [c | c : _ <- rows req "rates"]

assemble :: DocReq -> Value
assemble req =
  object $
    ["schema" .= schemaId, "rates" .= object (map census (rows req "rates"))]
      <> [fromString k .= fact req k | k <- header, '.' `notElem` k]
      <> [fromString g .= object (map (member g) ks) | (g, ks) <- groups]
 where
  dotted = [(takeWhile (/= '.') k, drop 1 (dropWhile (/= '.') k)) | k <- header, '.' `elem` k]
  groups = [(g, [f | (g', f) <- dotted, g' == g]) | g <- nub (map fst dotted)]
  member g f
    | g <> "." <> f == "run.rescanned" = fromString f .= flag req "run.rescanned"
    | otherwise = fromString f .= fact req (g <> "." <> f)

-- | One language's census under its report name.
census :: [Integer] -> Pair
census row = case row of
  [c, da, de, ua, ue, other, fold, self, saved] ->
    fromString (langName c)
      .= object
        [ "declared" .= object ["all" .= da, "exported" .= de]
        , "unmentioned" .= object ["all" .= ua, "exported" .= ue]
        , "vetoed" .= object ["other" .= other, "fold" .= fold, "self_text" .= self, "collision_saved" .= saved]
        ]
  _ -> "?" .= ()
