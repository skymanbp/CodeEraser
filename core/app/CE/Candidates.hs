-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | candidates.request handler (plan v2.33 W3; design booklet
-- docs/reference/algorithm-track.md §6): the whole T3 candidate pass
-- over integer facts. The measuring side sends every admitted unit as
-- its language, file, key, node count, line span and kind histogram,
-- each unit's structural shingle set, the fingerprint instances its
-- index holds and the near runs its T1/T2 extension found; this family
-- generates the sources S1 to S4 (CE.Candidates.Sources, CE.Candidates.T3),
-- applies the two admissible bounds (CE.Clone.Prefilter) and, when
-- asked, the exhaustive source S5, and answers the kept pairs and the
-- whole tally. Names, paths and kind strings never cross: every code is
-- request-local. No verdict here — the clone judgment (CE.Clone) judges
-- what this family keeps.
module CE.Candidates (respond) where

import CE.Candidates.Contract (CandReq (..), offence, overCap)
import CE.Candidates.Cost (keySource)
import CE.Candidates.Lsh (sets)
import CE.Candidates.Sources (Gen (..), generate)
import CE.Candidates.T3 (Answer (..), Tally (..), judgeT3)
import CE.Candidates.Units (columns)
import CE.Wire (family)
import CE.Wire.Result (resultLine)
import Data.Aeson (Value, (.=))
import Data.Bits (countTrailingZeros)
import qualified Data.ByteString.Char8 as B8
import qualified Data.IntMap.Strict as IM

-- | decode → cap → contract → the pass.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "candidates" reqId overCap offence (answer proto True) (answer proto False)

-- | The candidates.result object: the kept pairs, each source's
-- distinct pairs per language `[source, lang, n]` (source = bit index),
-- S4's bucket sizes `[size, n]`, the counts. Degraded: no pairs and a
-- zero tally — a request the core refused to judge keeps nothing — the
-- request tables still counted, and the reason.
answer :: String -> Bool -> CandReq -> B8.ByteString
answer proto degraded req =
  resultLine
    proto
    "candidates"
    (reqId req)
    [ "pairs" .= kept a
    , "raw" .= rawRows
    , "bandGroups" .= [[size, k] | (size, k) <- IM.toAscList (bandGroups g)]
    ]
    (zipWith (.=) countKeys countValues)
    (if degraded then Just "candidates_too_large" else Nothing)
 where
  countKeys =
    [ "units", "prints", "near", "union", "crossLanguage", "unowned", "selfPairs", "printHot", "bandHot"
    , "prunedSize", "prunedLabel", "survivors", "s5Windowed", "s5PrunedLabel", "s5Already", "s5New"
    ]
  countValues =
    map length [unitRows req, printRows req, nearRows req]
      <> [union t, crossLanguage t + cross g, unowned g, selfPairs g, printHot g, bandHot g]
      <> map ($ t) [prunedSize, prunedLabel, survivors, s5Windowed, s5PrunedLabel, s5Already, s5New]
  us = columns (unitRows req)
  g
    | degraded = Gen IM.empty [] 0 0 0 0 0 IM.empty
    | otherwise = generate us (sets (sigRows req)) (printRows req) (nearRows req)
  a
    | degraded = Answer [] (Tally 0 0 0 0 0 0 0 0 0) []
    | otherwise = judgeT3 (exhaustive req) us (unioned g)
  t = tally a
  sameKey = (fromInteger keySource, IM.fromList [(fromInteger l, fromInteger k) | [l, k] <- keyPairs a])
  rawRows =
    [ [countTrailingZeros bit, lang, k]
    | (bit, perLang) <- sortOnFst (sameKey : raw g)
    , (lang, k) <- IM.toAscList perLang
    ]
  sortOnFst = IM.toAscList . IM.fromList
