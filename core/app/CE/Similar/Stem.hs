-- | Porter's stemmer (plan v2.33 W6; moved from cli/src/similar/stem.rs,
-- every function below named after the Rust one it replaces) — M. F.
-- Porter, "An algorithm for suffix stripping", Program 14(3):130–137,
-- 1980 — the 1980 algorithm as the measuring side published it, over
-- ASCII lowercase only: a word carrying any other character returns
-- unchanged, so a non-English identifier piece is still a deterministic
-- term. A measure m over the [C](VC)^m[V] form and five tables of suffix
-- rules, each step taking the FIRST matching suffix whether or not its
-- condition then fires. One reading of the Rust is kept on purpose: in
-- step 1b the `ed` / `ing` rule counts as fired when the suffix MATCHED,
-- condition or not, so a vowel-less stem still reaches the tidy-up
-- (`bed` becomes `bede`) — the frozen behaviour the index was built with.
module CE.Similar.Stem (stem) where

import Data.Char (isAsciiLower)
import Data.List (isSuffixOf)

-- | The stem of one word. The Rust tests the UTF-8 byte length; a word
-- that is all ASCII lowercase has as many bytes as characters, and any
-- other word comes back unchanged either way.
stem :: String -> String
stem word
  | length word <= 2 || not (all isAsciiLower word) = word
  | otherwise = step5 (step4 (apply step3 (apply step2 (step1c (step1b (step1a word))))))

-- | A consonant is a letter other than a e i o u, and other than y when
-- a consonant precedes it.
isConsonant :: String -> Int -> Bool
isConsonant w i = case w !! i of
  c | c `elem` "aeiou" -> False
  'y' -> i == 0 || not (isConsonant w (i - 1))
  _ -> True

-- | m of the stem `take end w`: the number of VC sequences.
measure :: String -> Int -> Int
measure w end = count 0 (dropWhile id flags)
 where
  flags = map (isConsonant w) [0 .. end - 1]
  count m fs = case dropWhile not fs of
    [] -> m
    rest -> count (m + 1) (dropWhile id rest)

-- | *v*: the stem `take end w` contains a vowel.
hasVowel :: String -> Int -> Bool
hasVowel w end = any (not . isConsonant w) [0 .. end - 1]

-- | *d: the word ends with a double consonant.
doubleConsonant :: String -> Bool
doubleConsonant w = n >= 2 && w !! (n - 1) == w !! (n - 2) && isConsonant w (n - 1)
 where
  n = length w

-- | *o: the stem `take end w` ends cvc where the second c is not w, x or y.
cvc :: String -> Int -> Bool
cvc w end =
  end >= 3
    && isConsonant w (end - 1)
    && not (isConsonant w (end - 2))
    && isConsonant w (end - 3)
    && (w !! (end - 1)) `notElem` "wxy"

-- | Rule `(cond) S1 -> S2`: when the word ends with S1, replace it by S2
-- if the stem satisfies `cond`. Answers whether S1 matched at all (a
-- step stops at its first matching suffix) and the word after.
rule :: String -> String -> (String -> Int -> Bool) -> String -> (Bool, String)
rule s1 s2 cond w
  | not (s1 `isSuffixOf` w) = (False, w)
  | cond w stemLen = (True, take stemLen w <> s2)
  | otherwise = (True, w)
 where
  stemLen = length w - length s1

-- | The first rule whose suffix matches decides; none matching leaves the
-- word as it was (a step of `||`-chained rules).
firstOf :: [String -> (Bool, String)] -> String -> (Bool, String)
firstOf [] w = (False, w)
firstOf (r : rs) w = case r w of
  (True, w') -> (True, w')
  (False, _) -> firstOf rs w

step1a :: String -> String
step1a = snd . firstOf [rule s1 s2 always | (s1, s2) <- [("sses", "ss"), ("ies", "i"), ("ss", "ss"), ("s", "")]]
 where
  always _ _ = True

step1b :: String -> String
step1b w = case rule "eed" "ee" (\v n -> measure v n > 0) w of
  (True, w') -> w'
  (False, _) -> case firstOf [rule "ed" "" hasVowel, rule "ing" "" hasVowel] w of
    (False, _) -> w
    (True, w') -> tidy w'
 where
  tidy v
    | any (`isSuffixOf` v) ["at", "bl", "iz"] = v <> "e"
    | doubleConsonant v && last v `notElem` "lsz" = init v
    | measure v (length v) == 1 && cvc v (length v) = v <> "e"
    | otherwise = v

step1c :: String -> String
step1c w
  | n >= 1 && last w == 'y' && hasVowel w (n - 1) = init w <> "i"
  | otherwise = w
 where
  n = length w

-- | Steps 2 and 3: `(m > 0) S1 -> S2` rules in the paper's order, one text
-- per step as the Rust spells them (`s1>s2`, comma-separated).
apply :: String -> String -> String
apply rules = snd . firstOf [rule s1 (drop 1 s2) (\v n -> measure v n > 0) | (s1, s2) <- map (break (== '>')) (splitOn rules)]
 where
  splitOn s = case break (== ',') s of
    (a, _ : rest) -> a : splitOn rest
    (a, []) -> [a]

step2 :: String
step2 =
  "ational>ate,tional>tion,enci>ence,anci>ance,izer>ize,abli>able,alli>al,\
  \entli>ent,eli>e,ousli>ous,ization>ize,ation>ate,ator>ate,alism>al,\
  \iveness>ive,fulness>ful,ousness>ous,aliti>al,iviti>ive,biliti>ble"

step3 :: String
step3 = "icate>ic,ative>,alize>al,iciti>ic,ical>ic,ful>,ness>"

-- | Step 4 suffixes, `(m > 1) S -> ""`, longer-before-shorter where one is
-- the other's tail; `ion` additionally needs the stem to end in s or t.
step4 :: String -> String
step4 = snd . firstOf [rule s "" (cond s) | s <- words suffixes]
 where
  suffixes = "al ance ence er ic able ible ant ement ment ent ion ou ism ate iti ous ive ize"
  cond s v n = measure v n > 1 && (s /= "ion" || (n >= 1 && (v !! (n - 1)) `elem` "st"))

-- | Step 5a drops a final e when m > 1, or when m = 1 and the stem is not
-- *o; step 5b singles a final double l when m > 1.
step5 :: String -> String
step5 = fiveB . fiveA
 where
  fiveA w
    | not (null w) && last w == 'e' && (m > 1 || (m == 1 && not (cvc w (n - 1)))) = init w
    | otherwise = w
   where
    n = length w
    m = measure w (n - 1)
  fiveB w
    | length w >= 2 && last w == 'l' && doubleConsonant w && measure w (length w) > 1 = init w
    | otherwise = w
