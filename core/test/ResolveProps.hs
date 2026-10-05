{-# LANGUAGE OverloadedStrings #-}

-- | The resolve family's property legs (plan v2.33 wave W2a; on text
-- since W2-text): the contract refuses a malformed request by name, the
-- cap counts the request's text, the answers stand on the tree alone —
-- not on the order of the sites, not on a file the walk did not hold —
-- and a response file the request lacks is asked for before any answer
-- is given. The seeded cases are ReferenceResolve's.
module ResolveProps (battery) where

import CE.Resolve (respond)
import CE.Resolve.Contract (overCap, requestSize)
import CE.Resolve.Cost (Reason (..), langPy, resolveCap)
import CE.Resolve.Request (CReq (..), ResolveReq (..), Site (..))
import Data.Aeson (Value (..), decodeStrict, encode, object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as BL
import Data.Aeson.Types (parseMaybe, parseJSON)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)
import ReferenceResolveGen
import WireHarness (field, refusedBy, runLegs, setKey)

battery :: IO Bool
battery = runLegs names probes
 where
  names =
    [ "resolve: reversing the sites reverses the answers"
    , "resolve: an unwalked file is never a target"
    , "resolve: counts name the sites and the files"
    , "resolve: no compile database, no forced arcs"
    , "resolve: a response file the request lacks is wanted, the answers wait"
    , "resolve: walked paths out of path order are refused by name"
    , "resolve: an origin the walk holds is refused by name"
    , "resolve: a site in a language this family does not hold is refused"
    , "resolve: a site whose file is out of range is refused"
    , "resolve: a database probe out of range is refused"
    , "resolve: the cap counts the request's text"
    ]
  probes =
    [ all reversible (take 40 cases)
    , unwalkedIsNoTarget
    , all counted (take 40 cases)
    , all (\c -> (reply' (request c) >>= (`field` "forced")) == Just (toJSON ([] :: [Value]))) (take 20 cases)
    , wantedFirst
    , refused (setKey "files" (toJSON ["b.py", "a.py" :: String]) base) "file 1: not after the path before it"
    , refused (setKey "origins" (toJSON [firstFile]) base) "origin 0: a walked file"
    , refused (setKey "sites" (toJSON [[toJSON (3 :: Int), toJSON (0 :: Int), toJSON (0 :: Int), toJSON ("x" :: String)]]) base) "language this family does not resolve"
    , refused (setKey "sites" (toJSON [[toJSON langPy, toJSON (0 :: Int), toJSON (9999 :: Int), toJSON ("x" :: String)]]) base) "kind or file out of range"
    , refused (setKey "c" (object ["dbs" .= [[toJSON ("" :: String), toJSON (3 :: Int), toJSON ("x" :: String)]]]) base) "c.db 0: probe out of range"
    , capLeg
    ]
  base = request firstCase
  firstFile = case cFiles firstCase of
    (f : _) -> f
    [] -> ""
  refused r want = refusedBy respond r want

firstCase :: Case
firstCase = case cases of
  (c : _) -> c
  [] -> error "no seeded case"

reply :: Value -> Maybe Value
reply r = either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode r)))

reply' :: Value -> Maybe (KM.KeyMap Value)
reply' r = case reply r of
  Just (Object o) -> Just o
  _ -> Nothing

-- | Reversed sites, reversed answers.
reversible :: Case -> Bool
reversible c = fmap reverse (reply (request c) >>= answers) == (reply (request c {cSites = reverse (cSites c)}) >>= answers)

-- | A Python site whose only match is a file the walk did not hold.
unwalkedIsNoTarget :: Bool
unwalkedIsNoTarget =
  let c walkedM = firstCase {cFiles = ["a/m.py" | walkedM] <> ["a/n.py"], cExtra = ["a/m.py" | not walkedM], cSites = [(langPy, 0, "a/n.py", ".m")]}
      ask x = reply (request x) >>= answers
   in ask (c True) == Just [RFile "a/m.py" 1] && ask (c False) == Just [RUnres OutOfScope]

-- | counts.sites and counts.files agree with the request.
counted :: Case -> Bool
counted c = case reply' (request c) >>= (`field` "counts") of
  Just (Object o) -> KM.lookup "sites" o == Just (toJSON (length (cSites c))) && KM.lookup "files" o == Just (toJSON (length (cFiles c)))
  _ -> False

-- | A database whose entry names a response file: the first reply wants
-- it and answers nothing; with its text carried, nothing is wanted, the
-- answers come, and the expansion names it among the responses.
wantedFirst :: Bool
wantedFirst = asked ([] :: [[Value]]) == (["r.rsp"], 0) && asked [[toJSON ("r.rsp" :: String), toJSON ("-Ia" :: String)]] == ([], length (cSites firstCase))
 where
  db = object ["dbs" .= [[toJSON ("" :: String), toJSON (0 :: Int), toJSON ("compile_commands.json" :: String)]], "json" .= [[toJSON ("compile_commands.json" :: String), toJSON [object ["directory" .= ("" :: String), "file" .= ("a/z.c" :: String), "arguments" .= ["cc", "@r.rsp" :: String]]]]]]
  asked :: [[Value]] -> ([String], Int)
  asked texts =
    let r = reply' (setKey "c" (withTexts texts) (request firstCase))
        list k = fromMaybe [] (r >>= KM.lookup k >>= parseMaybe parseJSON)
     in (list "wanted" :: [String], length (list "results" :: [Value]))
  withTexts texts = case db of
    Object o -> Object (KM.insert "responses" (toJSON texts) o)
    v -> v

-- | The size counts one per row and one per character: 2^14 sites of
-- 2^14 - 1 characters sit exactly at the cap; one empty walked path more
-- is over it. The specifier is one shared string, so the leg walks it
-- without holding the cap's worth of characters.
capLeg :: Bool
capLeg = requestSize (req []) == resolveCap && not (overCap (req [])) && overCap (req [""])
 where
  spec = replicate (2 ^ (14 :: Int) - 1) 'a'
  req files = ResolveReq (Number 1) files [] (replicate (2 ^ (14 :: Int)) (Site langPy 0 0 spec Nothing)) M.empty Nothing [] [] [] [] [] [] [] [] [] (CReq "" [] [] [] [] []) Nothing
