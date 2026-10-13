-- | scan/1's fence channel (6.4.0, O33): the `knobsFence` value as it
-- rode, read into one of three states and judged by the verdict
-- road's own Maybe-equality (CE.Verdict digestDrift), so the three
-- roads — check, scan, guard — agree by construction. Its own module
-- since the scan family's file reached the size wall it gates.
module CE.Scan.Fence (Fence (..), drifted, fenceOffence, readFence) where

import Data.Aeson
import Data.Aeson.Types (parseEither)

-- | null = no committed baseline (unfenced); [current, recorded] = the
-- digest this run declares and the one the baseline recorded, each null
-- for "the shipped default". The key itself is required since 9.0.0
-- (CE.Scan.Contract).
data Fence = Unfenced | Fence (Maybe Integer) (Maybe Integer)

readFence :: Value -> Either String Fence
readFence Null = Right Unfenced
readFence v = case parseEither parseJSON v of
  Right [a, b] | all (maybe True (>= 0)) [a, b] -> Right (Fence a b)
  _ -> Left "knobsFence: malformed (need null or [current,recorded])"

-- | The boundary refusal: a value that is neither null nor a
-- well-formed pair names itself.
fenceOffence :: Value -> Maybe String
fenceOffence = either Just (const Nothing) . readFence

drifted :: Fence -> Bool
drifted (Fence a b) = a /= b
drifted Unfenced = False
