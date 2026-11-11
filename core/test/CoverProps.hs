-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | cover/1's battery (v2.35 imports lane, R2): the two properties that
-- define the coverage rule on every generated pair, the rule against a
-- second spelling (ReferenceCover: token-position sets) over an
-- exhaustive small universe, the named cases — R2-11, the low-diversity
-- run that hid a diverse one, the shared cover of a same-file pair, the
-- 2 × 2 family — every contract refusal by name, and the reply's shape.
module CoverProps (battery) where

import CE.Dedup.Cost (coverRowCap)
import CE.Dedup.Cover (respond)
import CE.Dedup.Cover.Rule (Class (..), Run (..), classify)
import Data.Aeson
import Data.List (tails)
import ReferenceCover (Row, keptInOrder, refClasses, uncoveredWhenKept)
import WireHarness (fieldsOf, refusedBy, runLegs, setKey, tabledRequest)

battery :: IO Bool
battery =
  runLegs
    [ "cover: completeness and non-redundancy on every generated pair"
    , "cover: the rule agrees with the token-position spelling on the small universe"
    , "cover: R2-11's two runs on one line pair are both kept"
    , "cover: a low-diversity run never suppresses a diverse one"
    , "cover: a same-file pair shares one cover (three copies, two rows; a periodic list, one)"
    , "cover: the 2 x 2 family keeps three of four links and names every region"
    , "cover: every contract offence is refused by name"
    , "cover: the reply partitions the rows, counts them and echoes t and d"
    ]
    [properties, agreement, r211, lowNeverHides, sharedCover, family2x2, refusals, replyShape]

-- | The rule's codes in row order (0 reported, 1 suppressed, 2 dropped).
codes :: Integer -> Integer -> [Row] -> [Int]
codes t d rows = map code (classify t d [Run a b a0 b0 n k | (a, b, a0, b0, n, k) <- rows])
 where
  code c = case c of Reported -> 0; Suppressed -> 1; Dropped -> 2

-- | The small universe: one pair of two files and one same-file pair,
-- every set of up to three runs over a short grid with distinct (a0, b0),
-- under t in 1..3 and d in 1..2.
universe :: [(Integer, Integer, [Row])]
universe = [(t, d, rows) | t <- [1 .. 3], d <- [1, 2], rows <- sets]
 where
  sets = filter fresh (upTo3 cross <> upTo3 self)
  cross = [(0, 1, a0, b0, n, k) | a0 <- [0 .. 2], b0 <- [0 .. 2], n <- [1 .. 3], k <- [1, n]]
  self = [(2, 2, a0, b0, n, k) | a0 <- [0 .. 2], b0 <- [a0 + 1 .. 4], n <- [1 .. b0 - a0], k <- [1, n]]
  upTo3 xs = [[]] <> [[x] | x <- xs] <> [[x, y] | x : ys <- tails xs, y <- ys] <> [[x, y, z] | x : ys <- tails xs, y : zs <- tails ys, z <- zs]
  fresh rows = let ks = [(a0, b0) | (_, _, a0, b0, _, _) <- rows] in length ks == length (dedupe ks)
  dedupe = foldr (\x acc -> if x `elem` acc then acc else x : acc) []

properties :: Bool
properties = all holds universe
 where
  holds (t, d, rows) =
    let kept = [r | (r, c) <- zip rows (codes t d rows), c /= 2]
        complete = and [c /= 2 || both (< t) (uncoveredWhenKept kept r) | (r, c) <- zip rows (codes t d rows)]
        lean = all (\(_, (na, nb)) -> max na nb >= t) (keptInOrder t d rows)
     in complete && lean && length (keptInOrder t d rows) == length kept
  both p (x, y) = p x && p y

agreement :: Bool
agreement = all (\(t, d, rows) -> codes t d rows == refClasses t d rows) universe

r211 :: Bool
r211 = codes 50 7 [(0, 1, 0, 0, 60, 20), (0, 1, 200, 300, 52, 20)] == [0, 0]

-- | The luarocks appearing row reduced: a long run of three distinct
-- tokens holding a diverse one inside it. The diverse one is reported;
-- the long one keeps 60 tokens of its own and is suppressed, not dropped.
lowNeverHides :: Bool
lowNeverHides = codes 50 7 [(0, 1, 0, 0, 120, 3), (0, 1, 30, 30, 60, 20)] == [1, 0]

-- | Three copies of 60 tokens in one file (gaps between them) give two
-- rows; a periodic list of period 10 over 150 tokens gives one.
sharedCover :: Bool
sharedCover =
  codes 50 7 [(3, 3, 0, 100, 60, 20), (3, 3, 0, 200, 60, 20), (3, 3, 100, 200, 60, 20)] == [0, 0, 2]
    && codes 50 7 [(3, 3, 0, g, n, 10) | (g, n) <- [(50, 50), (60, 60), (70, 70), (80, 70), (90, 60), (100, 50)]] == [2, 2, 0, 2, 2, 2]
    && codes 50 7 [(3, 3, 102, 255, 153, 10), (3, 3, 102, 272, 152, 10)] == [0, 2]

-- | File 4 holds two copies (0, 100), file 5 two (0, 100): four links.
family2x2 :: Bool
family2x2 = classes == [0, 0, 0, 2] && all named [(4, 0), (4, 100), (5, 0), (5, 100)]
 where
  rows = [(4, 5, a0, b0, 60, 20) | a0 <- [0, 100], b0 <- [0, 100]]
  classes = codes 50 7 rows
  kept = [r | (r, c) <- zip rows classes, c == 0]
  named (f, s) = or [(a == f && a0 == s) || (b == f && b0 == s) | (a, b, a0, b0, _, _) <- kept]

req :: Integer -> Integer -> [[Integer]] -> Value
req t d rows = setKey "minDistinct" (toJSON d) (setKey "minTokens" (toJSON t) (tabledRequest "9.1.0" "cover.request" [("runs", rows)]))

refusals :: Bool
refusals =
  refusedBy respond (req 50 7 (replicate (fromInteger coverRowCap + 1) [0, 1, 0, 0, 1, 1])) ("rows over the cap of " <> show coverRowCap)
    && refusedBy respond (req 0 7 []) "minTokens: below 1"
    && refusedBy respond (req 50 0 []) "minDistinct: below 1"
    && and (zipWith (ows why -> refusedBy respond (req 50 7 rows) why) offending offences)
 where
  offending =
    [ [[0, 1, 0, 0, 1]]
    , [[0, 1, -1, 0, 1, 1]]
    , [[1, 0, 0, 0, 1, 1]]
    , [[0, 1, 0, 0, 0, 1]]
    , [[0, 1, 0, 0, 2, 3]]
    , [[0, 1, 0, 0, 2, 0]]
    , [[2, 2, 0, 3, 4, 1]]
    , [[0, 1, 5, 0, 1, 1], [0, 1, 5, 0, 1, 1]]
    , [[0, 2, 0, 0, 1, 1], [0, 1, 0, 0, 1, 1]]
    ]
  offences =
    [ "runs 0: malformed run"
    , "runs 0: negative element"
    , "runs 0: fileA after fileB"
    , "runs 0: len below 1"
    , "runs 0: distinct outside 1..len"
    , "runs 0: distinct outside 1..len"
    , "runs 0: a same-file run overlaps itself"
    , "runs 1: not strictly ascending"
    , "runs 1: not strictly ascending"
    ]

replyShape :: Bool
replyShape =
  fieldsOf respond (req 50 7 rows) ["reported", "suppressed", "dropped", "counts", "minTokens", "minDistinct", "degraded"]
    == Just
      [ Just (toJSON [1 :: Int, 2])
      , Just (toJSON [0 :: Int])
      , Just (toJSON [3 :: Int])
      , Just (object ["rows" .= (4 :: Int), "reported" .= (2 :: Int), "suppressed" .= (1 :: Int), "dropped" .= (1 :: Int)])
      , Just (toJSON (50 :: Int))
      , Just (toJSON (7 :: Int))
      , Just (Bool False)
      ]
 where
  rows = [[0, 1, 0, 0, 120, 3], [0, 1, 30, 30, 60, 20], [6, 6, 0, 100, 60, 20], [6, 6, 10, 110, 40, 20]]
