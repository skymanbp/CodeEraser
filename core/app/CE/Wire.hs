-- | The judgment-family wire skeleton (ADR-008 P3 ratchet
-- repayment: the sixth family recloned the respond cascade and the
-- ascending checker for the third time — the repo's own dedup gate
-- caught its author a tenth time, so the skeleton itself becomes
-- the single authority). A family says WHAT — its caps, offences
-- and replies — as data; this module owns the HOW of answering one
-- request line. CE.Verdict keeps its own cascade: its parsed
-- baseline threads through cap AND offence, a shape this skeleton
-- deliberately does not grow to cover.
module CE.Wire (Family (..), applyRows, ascendingOn, pick, respondWith, notAscending) where

import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import Data.Foldable (asum)

-- | One family's bindings for the shared cascade.
data Family req = Family
  { famName :: String
  -- ^ decode-error prefix ("clone: ...")
  , famId :: req -> Value
  , famOverCap :: req -> Bool
  , famOffence :: req -> Maybe String
  , famDegraded :: req -> B8.ByteString
  , famJudged :: req -> B8.ByteString
  }

-- | decode → cap check (a complete degraded reply, never a
-- truncated one) → boundary contract (error/contract naming the
-- first offender) → judge.
respondWith ::
  (FromJSON req) =>
  Family req ->
  B8.ByteString ->
  Either (Maybe Value, String, String) B8.ByteString
respondWith fam line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", famName fam <> ": " <> e)
  Right req
    | famOverCap fam req -> Right (famDegraded fam req)
    | Just why <- famOffence fam req -> Left (Just (famId fam req), "contract", why)
    | otherwise -> Right (famJudged fam req)

-- | Strictly-ascending row check with the family's label vocabulary
-- (list Ord is lexicographic) — one comparison, four families.
notAscending :: (Ord a) => String -> Int -> (a, a) -> Maybe String
notAscending what i (prev, cur)
  | prev < cur = Nothing
  | otherwise = Just (what <> " " <> show i <> ": not strictly ascending")

-- | Whole-table ascending pass on a PROJECTION of each row — the
-- identity prefix for docdup pairs (take 2), the code for scan
-- grades (take 1), the whole row for clone pairs and graph edges
-- (id). One zipWith, five call sites: the review-repair batch's own
-- ratchet bite was the projection lambda cloning across families.
ascendingOn :: (Ord b) => String -> (a -> b) -> [a] -> Maybe String
ascendingOn what proj rows =
  asum
    ( zipWith
        (\i (p, c) -> notAscending what i (proj p, proj c))
        [1 :: Int ..]
        (zip rows (drop 1 rows))
    )

-- | Fold [code, value] rows through a setter table; rows whose code
-- the table does not own fall through untouched (validation already
-- bounded every code). Promoted from CE.Verdict when the seventh
-- family recloned it verbatim (the twelfth ratchet bite).
applyRows :: [(Integer, Integer -> a -> a)] -> [[Integer]] -> a -> a
applyRows setters = flip (foldl' step)
 where
  step k row = case row of
    [code, v] | Just set <- lookup code setters -> set v k
    _ -> k

-- | Last [code, value] match or the default (later rows win — the
-- applyRows fold order, though validation's ascending rule already
-- forbids duplicate codes within one table).
pick :: [[Integer]] -> Integer -> Integer -> Integer
pick rows code dflt = last (dflt : [v | [c, v] <- rows, c == code])
