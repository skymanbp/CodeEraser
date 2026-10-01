-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The console form of the documents (plan v2.32 step 5; design
-- booklet docs/reference/authority-track.md §6): every family's
-- `lines` and `exit`. Over the seeded requests of every family, in
-- both languages, and over the document golden's requests: every line
-- holds as many `{}` holes as references, every reference is a stated
-- class inside its ranges, every template of every catalogue is
-- reached and none is missed or misfilled; every family answers lines
-- and a veto for its empty request and past the cap; a language other
-- than 0 or 1 is refused by name; the veto is the face's rule restated
-- from the request and the document; the core's `{:.1}` is Rust's on
-- the halfway cases; the golden speaks both languages; and a request
-- that leaves out a family's optional facts and tables assembles the
-- same document.
module DocumentProps5 (battery) where

import CE.Document (families, respond)
import CE.Document.Contract (DocFamily (..), DocReq, Spec (..), Table (..), docRowCap)
import CE.Text (Line (..), Piece (..), fixed, holes, langOf)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import Data.Foldable (toList)
import Data.List (isPrefixOf, nub)
import Data.Maybe (fromMaybe, isJust)
import qualified Data.Set as S
import DocumentGen (requests)
import DocumentGen4 (requests4)
import DocumentGen5 (requests5)
import DocumentGen6 (requests6)
import DocumentHarness (at, documentOf, emptyRequest, int, ints, items, path, refsHeld)
import WireHarness (field, refusedBy, replyObjWith, runLegs, setKey)

battery :: IO Bool
battery = do
  raw <- B8.readFile "../contracts/fixtures/document/golden.ndjson"
  -- the golden's requests the contract accepts (its refusals have no lines)
  let golden = [r | l <- B8.lines raw, Just r <- [decodeStrict l], at r "type" == Just "document.request", isJust (replyObjWith respond r)]
      said = spoken (seeded <> golden)
  runLegs
    [ "every line holds as many {} holes as references, in both languages"
    , "every reference in a line is a stated class inside its ranges"
    , "every template of every family's catalogue is reached, none missed or misfilled"
    , "every family answers lines and a veto for its empty request and past the cap"
    , "a language other than 0 or 1 is refused by name"
    , "the veto is the face's rule, restated, on every seeded and golden request"
    , "{:.1} is Rust's on the halfway cases, the signs and the zero denominator"
    , "the golden speaks both languages for every family with a console"
    , "leaving out the optional facts and tables leaves the document as it was"
    ]
    [ all holed said
    , refsHeld [(f, r, toJSON (map wire ls)) | (f, r, ls) <- said]
    , covered said
    , all answers families
    , all refusesLang families
    , all vetoed (seeded <> golden)
    , formatted
    , bilingual golden
    , all idle (seeded <> golden) && any optionals seeded
    ]

-- | Every seeded request of every family.
seeded :: [Value]
seeded = concat [gen (dfName f) | f <- families]
 where
  gen n
    | n `elem` words "arch query rules flow merge" = requests n
    | n `elem` words "guard audit" = requests5 n
    | n `elem` words "scan dedup clone clone-units docdup erase erase-trail churn trend similar" = requests6 n
    | otherwise = requests4 n

-- | Each request in both languages, with its family and lines.
spoken :: [Value] -> [(DocFamily, Value, [Line])]
spoken rs =
  [ (f, r', dfLines f (langOf (Just lang)) q)
  | r <- rs
  , Just f <- [familyOf r]
  , lang <- [0, 1]
  , let r' = setKey "lang" (toJSON lang) r
  , Success q <- [fromJSON r' :: Result DocReq]
  ]

familyOf :: Value -> Maybe DocFamily
familyOf r = case at r "family" >>= \v -> case fromJSON v of Success n -> Just n; _ -> Nothing of
  Just n -> lookup (n :: String) [(dfName f, f) | f <- families]
  Nothing -> Nothing

wire :: Line -> Value
wire (Line s (Piece t r _)) = toJSON (toJSON s : toJSON t : r)

holed :: (DocFamily, Value, [Line]) -> Bool
holed (_, _, ls) = all (\(Line _ (Piece t r _)) -> holes t == length r) ls

-- | Each text catalogue once, under the first family spoken from it
-- (rules shares query's, the unit listing clone's and the trail
-- erase's): twenty.
catalogues :: [(String, [String])]
catalogues = [(dfName f, map fst (dfText f)) | f <- families, not (null (dfText f)), owner f == dfName f]

owner :: DocFamily -> String
owner f = concat (take 1 [dfName g | g <- families, dfText g == dfText f])

-- | No key missed or misfilled (`!key`), no key twice in a catalogue,
-- and every key reached by a family that reads it.
covered :: [(DocFamily, Value, [Line])] -> Bool
covered xs = length catalogues == 20 && not (any (("!" `isPrefixOf`) . snd) (S.toList used)) && all reached catalogues && all unique catalogues
 where
  used = S.fromList [(owner f, k) | (f, _, ls) <- xs, Line _ (Piece _ _ ks) <- ls, k <- ks]
  reached (n, ks) = all (\k -> S.member (n, k) used) ks
  unique (_, ks) = length ks == length (nub ks)

-- | The empty request and one past the cap both answer `lines` (an
-- array of `[stream, text, ref…]`) and `exit.fail` (a bool).
answers :: DocFamily -> Bool
answers f = all spoke (emptyRequest f : over)
 where
  over = [setKey "rows" (object [Key.fromString (tName t) .= replicate (fromInteger docRowCap + 1) [0 :: Integer]]) (emptyRequest f) | t <- take 1 (spTables (dfSpec f))]
  spoke r = case replyObjWith respond (setKey "lang" (toJSON (1 :: Int)) r) of
    Just o -> shaped (field o "lines") && (field o "exit" >>= (`at` "fail")) `elem` [Just (Bool True), Just (Bool False)]
    Nothing -> False
  shaped v = case v of
    Just (Array ls) -> all line' (toList ls)
    _ -> False
  line' l = case l of
    Array xs | (s : String _ : _) <- toList xs -> s `elem` [Number 0, Number 1]
    _ -> False

refusesLang :: DocFamily -> Bool
refusesLang f = all (\c -> refusedBy respond (setKey "lang" (toJSON c) (emptyRequest f)) "document: lang is not 0 or 1") [2, -1 :: Integer]

-- | The veto, restated from the request's facts and the document.
vetoed :: Value -> Bool
vetoed r = case (familyOf r, replyObjWith respond r) of
  (Just f, Just o) -> (field o "exit" >>= (`at` "fail")) == Just (Bool (rule (dfName f) (documentOf r)))
  _ -> False
 where
  fact k = int (path ["facts", k] r)
  -- a family no rule names never vetoes
  rule n d = maybe False (\veto -> veto (fromMaybe Null d)) (lookup n rules)
  rules =
    [ ("rules", \x -> and [null (items x "errors"), at x "degraded" == Just Null, fact "violations" > 0])
    , ("flow", \x -> and [fact "check" == 1, fact "deny" == 1, int (path ["counts", "judged"] x) > 0])
    , ("check", const (fact "fail" == 1))
    , ("deadcode", \x -> fact "check" == 1 && (not (null (items x "dead")) || at x "degraded" /= Just Null))
    , ("audit", const (and [fact "git" == 1, fact "unreadable" == 0, tomb || dups]))
    , ("scan", \x -> not (null (items x "failed")))
    , ("dedup", const (fact "check" == 1 && fact "fail" == 1))
    , ("docdup", \x -> fact "check" == 1 && not (null (items x "dups")))
    , ("erase", \x -> fact "check" == 1 && int (path ["counts", "eraseable"] x) > 0)
    , ("erase-trail", \x -> not (null (items x "unreadable")))
    , ("trend", \x -> path ["judgment", "fail"] x == Just (Bool True) || not (null (items x "failed")))
    ]
  tomb = case ints (path ["rows", "tomb"] r) of
    [[1, _, _, _, _, _, 3, 1, unread, bounded]] -> unread + bounded == 0
    _ -> False
  dups = fact "mode" == 3 && case ints (path ["rows", "dups"] r) of
    [[_, 1]] -> True
    _ -> False

-- | Rust `format!("{:.d}", v as f64 / d as f64)`, read off rustc
-- 1.94.1 (the subrepo's ROI leg reads the same against the golden).
formatted :: Bool
formatted =
  map (\v -> fixed 1 v 1000) [250, 1250, 350, 2500, 1350, 999, 1000] == ["0.2", "1.2", "0.3", "2.5", "1.4", "1.0", "1.0"]
    && [fixed 1 (-1) 1000, fixed 1 1 0, fixed 1 (-1) 0, fixed 1 0 0, fixed 0 5 2, fixed 2 1 3] == ["-0.0", "inf", "-inf", "NaN", "2", "0.33"]

-- | The request without its family's optional facts and tables (the
-- console's alone) is accepted and assembles the same document.
idle :: Value -> Bool
idle r = null names || (isJust (documentOf r') && documentOf r' == documentOf r)
 where
  names = maybe [] (spOptional . dfSpec) (familyOf r)
  r' = foldr without r ["facts", "rows"]
  without t v = case at v t of
    Just (Object o) -> setKey t (Object (foldr (KM.delete . Key.fromString) o names)) v
    _ -> v

optionals :: Value -> Bool
optionals r = maybe False (not . null . spOptional . dfSpec) (familyOf r)

-- | The golden holds a `lang` 1 request of every family with a console
-- (the graph screen has none).
bilingual :: [Value] -> Bool
bilingual golden = all (\f -> any (\r -> (dfName <$> familyOf r) == Just (dfName f) && at r "lang" == Just (Number 1)) golden) speaking
 where
  speaking = [f | f <- families, dfName f /= "graphscreen"]
