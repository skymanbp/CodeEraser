-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Split from CE.Verdict.Wire at the 300-line core gate: a request
-- is checked row by row against its node universe; a baseline is a
-- DOCUMENT, read whole under the same row discipline.
module CE.Verdict.Baseline (parseBaseline) where

import CE.Verdict.Ratchet (Baseline (..))
import CE.Verdict.Table (ascendingBy, label)
import Data.Aeson
import qualified Data.Aeson.Types as AT
import Data.Foldable (asum)

-- | The one reader of ce-baseline.json bytes: null = establish;
-- otherwise {continuous, discrete} with the same row discipline as
-- the live tables, plus (2.14.0, additive) the optional frozen
-- softLine — absent or null on a pre-v0.6 file, and the size axis
-- then falls back to the sizeCeil knob. Entities are NOT
-- range-checked against the tier universe — a baseline may outlive
-- the files it measured.
parseBaseline :: Value -> Either String (Maybe Baseline)
parseBaseline Null = Right Nothing
parseBaseline v = case AT.parse bl v of
  AT.Error e -> Left ("baseline: " <> e)
  AT.Success (cont, disc, soft, digest) ->
    case asum
      [ asum (zipWith contShape [0 :: Int ..] cont)
      , ascendingBy "baseline.continuous" 2 cont
      , ascendingBy "baseline.discrete" 1 (map pure disc)
      , softShape soft
      ] of
      Just why -> Left why
      Nothing -> Right (Just (Baseline cont disc soft digest))
 where
  bl = withObject "baseline" $ \o ->
    (,,,) <$> o .: "continuous" <*> o .: "discrete" <*> o .:? "softLine" <*> o .:? "knobsDigest"
  contShape i row = case row of
    [_, code, _]
      | any (< 0) row -> Just (label "baseline.continuous" i <> "negative field")
      | code > 6 -> Just (label "baseline.continuous" i <> "unknown metric code")
      | otherwise -> Nothing
    _ -> Just (label "baseline.continuous" i <> "malformed row")
  softShape Nothing = Nothing
  softShape (Just s)
    | s < 1 || s >= 18446744073709551616 = Just "baseline.softLine: outside 1..u64"
    | otherwise = Nothing
