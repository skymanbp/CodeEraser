{-# LANGUAGE OverloadedStrings #-}

-- | The definition package's battery (plan v2.32 step 1; design
-- booklet docs/reference/authority-track.md §4): the relations the
-- measuring side asserted over its own tables before the core held
-- them — language codes and extensions, the judged mask, the dead
-- nest-only entry, the slot sets, the flow name tables and their
-- dispatch — now asserted where the tables live, plus the digest as a
-- function of the data and the `tables/1` reply as the package.
module LangProps (battery) where

import qualified CE.Handshake as Handshake
import CE.Lang (allTables, digestOf, languages)
import CE.Lang.Spec
import CE.Lang.Spec.Flow
import CE.Tables (package, respond, tablesDigest)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import Data.Bits (shiftL, (.|.))
import qualified Data.ByteString.Char8 as B8
import Data.Char (isAscii, isSpace)
import qualified Data.Map.Strict as M
import Data.Maybe (isJust, mapMaybe)
import WireHarness (field, refusedBy, replyObjWith, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "language codes run 0..21 in order"
    , "no extension names two languages"
    , "the judged mask is 0x17847F and the tables are the judged languages"
    , "the flow-judged mask is 1540127, inside the judged set"
    , "no nest-only kind is a unit kind"
    , "the slot sets are disjoint and the name pairs single"
    , "flow name tables are plain and single"
    , "every flow table has containers, exits and names; the C family's for"
    , "flow tables answer the judged code languages; C and C++ part on noreturn alone"
    , "the digest is a function of the data"
    , "tables/1 answers the package and the hello's digest"
    , "tables/1 refuses a key beyond the envelope by name"
    ]
    [codes, extensions, judged, flowJudged, nestOnly, slots, flowNames, flowShape, dispatch, digest, reply, refusal]

codes :: Bool
codes = map lgCode languages == [0 .. 21]

extensions :: Bool
extensions = unique (concatMap lgExts languages)

judged :: Bool
judged =
  mask == 0x17847F && map ltName allTables == [lgName l | l <- languages, lgJudged l]
 where
  mask = foldl' (.|.) (0 :: Integer) [1 `shiftL` lgCode l | l <- languages, lgJudged l]

-- | The flow family's verdict languages (the ten whose flow precision
-- exams passed): a flow-judged language is a judged one.
flowJudged :: Bool
flowJudged =
  mask == 1540127 && and [lgJudged l | l <- languages, lgFlowJudged l]
 where
  mask = foldl' (.|.) (0 :: Integer) [1 `shiftL` lgCode l | l <- languages, lgFlowJudged l]

-- | A nest-only kind that is also a unit kind could never fire.
nestOnly :: Bool
nestOnly = all dead (map ltScan allTables)
 where
  dead s = not (any (`elem` scFnKinds s) (scCocNestOnlyKinds s))

-- | Expression, type, part, other and statement kinds part; the
-- statement and container lists are the slot table's own exactly when no
-- flow table gives them.
slots :: Bool
slots = and [ok t s | t <- allTables, Just s <- [ltSlot t]]
 where
  ok t s =
    unique (slExprKinds s <> slTypeKinds s <> slPartKinds s <> slOtherKinds s <> slStmtKinds s)
      && unique (slNameFields s)
      && (null (slStmtKinds s) && null (slContainerKinds s)) == isJust (ltFlow t)

flowNames :: Bool
flowNames = all names flows
 where
  names f =
    all single [flNoreturn f, flReturnCalls f, flDynamicNames f, flReceiverNames f, flDiscardNames f]
      && unique (flNoreturnAttrs f)
      && all (plain . snd) (flNoreturnAttrs f)
  single ns = unique ns && all plain ns
  plain n = not (null n) && all isAscii n && not (any isSpace n)

flowShape :: Bool
flowShape = all shaped flows && cStyle "c" && cStyle "cpp"
 where
  shaped f =
    not (null (flBlockKinds f))
      && not (null (flReturnKinds f) && null (flReturnCalls f))
      && not (null (flIdentKinds f))
  cStyle n = maybe False (any (not . null . lpUpdate) . flLoops) (flowOf n)

dispatch :: Bool
dispatch =
  all (\t -> isJust (ltFlow t) == (ltName t `notElem` ["markdown", "haskell", "html"])) allTables
    && flowOf "typescript" == flowOf "tsx"
    && fmap flNoreturn c /= fmap flNoreturn cpp
    && fmap (\f -> f {flNoreturn = []}) c == fmap (\f -> f {flNoreturn = []}) cpp
 where
  (c, cpp) = (flowOf "c", flowOf "cpp")

-- | Same data, same number twice; one name changed, another number.
digest :: Bool
digest = digestOf package == tablesDigest && digestOf package == digestOf package && digestOf renamed /= tablesDigest
 where
  renamed = case package of
    Object o -> Object (KM.insert "outputs" (toJSON ["renamed" :: String]) o)
    v -> v

reply :: Bool
reply = case replyObjWith respond request of
  Just o ->
    Object (foldr KM.delete o ["proto", "type", "id", "digest"]) == package
      && field o "type" == Just "tables.result"
      && field o "digest" == Just (toJSON tablesDigest)
      && hello == Just (toJSON tablesDigest)
  Nothing -> False
 where
  hello = do
    Object h <- decodeStrict (Handshake.respond "7.7.0" [] tablesDigest "0" (B8.pack "{\"proto\":\"7.7.0\"}"))
    field h "tablesDigest"

refusal :: Bool
refusal = refusedBy respond (setLang request) "tables: unexpected key lang"
 where
  setLang (Object o) = Object (KM.insert "lang" "c" o)
  setLang v = v

request :: Value
request = object ["proto" .= ("7.7.0" :: String), "type" .= ("tables.request" :: String), "id" .= (1 :: Int)]

flows :: [FlowSpec]
flows = mapMaybe ltFlow allTables

flowOf :: String -> Maybe FlowSpec
flowOf n = M.lookup n (M.fromList [(ltName t, f) | t <- allTables, Just f <- [ltFlow t]])

unique :: (Ord a) => [a] -> Bool
unique xs = length (nub' xs) == length xs
 where
  nub' = M.keys . M.fromList . map (\x -> (x, ()))
