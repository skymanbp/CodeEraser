-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The shared judgment-report envelope (plan v2.32 step 5), the
-- core's side of cli/src/report.rs `Pair` / `Report` / `envelope`: the
-- families whose report is the pairs a verdict bit names plus a counts
-- ledger — clone and docdup — state one shape, and the shape gives
-- their statement's pair table and facts, the verdict bit's check and
-- the document. A pair row is [a, b, three metrics, verdict (1 = a
-- hit)], in the order the judgment answered; a hit is two references
-- of the family's class and its metrics by name; the counts are the
-- universe, the measuring side's counters and the hits, in that order.
module CE.Document.Envelope (Envelope (..), envelopeOf, enveloped, envelopeStatement, verdicts) where

import CE.Document.Contract
import Data.Aeson (Value, object, (.=))
import Data.Aeson.Key (fromString)

data Envelope = Envelope
  { enSchema :: String
  , enHits :: String
  -- ^ the hits' key, also their count's name
  , enClass :: String
  -- ^ the reference class of a pair's ends
  , enUniverse :: (String, String)
  -- ^ the universe's count name and the range it is the size of
  , enMetrics :: [String]
  , enMeasured :: [String]
  }

-- | An envelope from its text: `schema hits class count range |
-- metric… | counter…`.
envelopeOf :: String -> Envelope
envelopeOf text = case map words (splitOn text) of
  [[schema, hits, cls, count, universe], metrics, measured] -> Envelope schema hits cls (count, universe) metrics measured
  _ -> error ("envelope text does not read: " <> text)
 where
  splitOn t = case break (== '|') t of
    (a, _ : rest) -> a : splitOn rest
    (a, []) -> [a]

-- | The statement lines the envelope needs: every counter a judged
-- fact, the pair table over the universe.
envelopeStatement :: Envelope -> String
envelopeStatement e =
  judgedFacts (enMeasured e)
    <> unwords (["rows pairs 6 judged", u, u] <> replicate 4 "-")
    <> "\n"
 where
  u = snd (enUniverse e)

-- | The verdict column is a bit.
verdicts :: DocReq -> Maybe String
verdicts req = codes req "pairs" 5 0 1

enveloped :: Envelope -> DocReq -> Value
enveloped e req =
  object
    [ "schema" .= enSchema e
    , fromString (enHits e) .= map hit hits
    , "counts" .= counted (fst (enUniverse e) : enMeasured e <> [enHits e]) ([range req (snd (enUniverse e))] <> map (fact req) (enMeasured e) <> [toInteger (length hits)])
    ]
 where
  hits = [r | r@[_, _, _, _, _, 1] <- rows req "pairs"]
  hit r = case r of
    a : b : metrics -> object (["a" .= ref (enClass e) [a], "b" .= ref (enClass e) [b]] <> zipWith (\k v -> fromString k .= v) (enMetrics e) metrics)
    _ -> object []
