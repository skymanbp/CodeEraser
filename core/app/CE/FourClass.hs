-- | fourclass.request handler: decode, machine-check the within-first
-- precondition at the boundary, classify, encode. The precondition —
-- no leftover added hash of a pair occurs among that same pair's
-- leftover removed hashes — is L1's within-file consumption rule seen
-- from this side; verifying its consequence turns a cross-language
-- assumption into a checked contract without duplicating the rule.
module CE.FourClass (respond) where

import CE.FourClass.Provenance (classify)
import CE.FourClass.Wire
import Data.Aeson (Value, eitherDecodeStrict)
import qualified Data.ByteString.Char8 as B8
import qualified Data.Set as S

-- | Left = (id to echo, error code, message) for the dispatcher's
-- error encoder; Right = the encoded fourclass.result line.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "fourclass: " <> e)
  Right req -> case violation (reqPairs req) of
    Just p ->
      Left
        ( Just (reqId req)
        , "contract"
        , "within-first violated: pair " <> show p
        )
    Nothing -> Right (encodeResult proto (classify req))

-- | First pair whose added and removed leftovers share a hash.
violation :: [Pair] -> Maybe Int
violation ps =
  case [pIdx p | p <- ps, shares p] of
    (p : _) -> Just p
    [] -> Nothing
 where
  shares p =
    not . S.null $
      S.intersection
        (S.fromList (map snd (pRem p)))
        (S.fromList (map snd (pAdd p)))
