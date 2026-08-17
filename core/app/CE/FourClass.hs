-- | fourclass.request handler: decode, machine-check the within-first
-- precondition at the boundary, classify, encode. The precondition —
-- no leftover added hash of a pair occurs among that same pair's
-- leftover removed hashes — is L1's within-file consumption rule seen
-- from this side; verifying its consequence turns a cross-language
-- assumption into a checked contract without duplicating the rule.
module CE.FourClass (respond) where

import CE.FourClass.Provenance (classify)
import CE.FourClass.Wire
import Control.Applicative ((<|>))
import Data.Aeson (Value, eitherDecodeStrict)
import qualified Data.ByteString.Char8 as B8
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | Left = (id to echo, error code, message) for the dispatcher's
-- error encoder; Right = the encoded fourclass.result line.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "fourclass: " <> e)
  Right req -> case violation (reqPairs req) of
    Just why -> Left (Just (reqId req), "contract", why)
    Nothing -> Right (encodeResult proto (classify req))

-- | First boundary offence: a duplicate pair index — Anchor's run
-- maps key on (pair, run), where M.fromList would silently DROP an
-- earlier duplicate's runs (M5-close review LOW; the Rust producer
-- enumerates, so this refuses drift, not traffic) — then the
-- within-first precondition (message bytes golden-pinned).
violation :: [Pair] -> Maybe String
violation ps = dup <|> within
 where
  dup = case M.keys (M.filter (> 1) (M.fromListWith (+) [(pIdx p, 1 :: Int) | p <- ps])) of
    (i : _) -> Just ("duplicate pair index: " <> show i)
    [] -> Nothing
  within = case [pIdx p | p <- ps, shares p] of
    (p : _) -> Just ("within-first violated: pair " <> show p)
    [] -> Nothing
  shares p =
    not . S.null $
      S.intersection
        (S.fromList [h | (_, h, _) <- concat (pRem p)])
        (S.fromList [h | (_, h, _) <- concat (pAdd p)])
