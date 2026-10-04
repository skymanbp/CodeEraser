{-# LANGUAGE OverloadedStrings #-}

-- | The resolve family's property legs (plan v2.33 wave W2a): the
-- contract refuses a malformed request by name, the cap counts every
-- table together, and the answers stand on the tree alone — not on the
-- numbering of the segment table, not on the order of the sites, not on
-- a file the walk did not hold. The seeded cases are ReferenceResolve's.
module ResolveProps (battery) where

import CE.Resolve (respond)
import CE.Resolve.Contract (overCap)
import CE.Resolve.Cost (Reason (..), langLua, langPy, resolveCap)
import CE.Resolve.Vocab (kindRequire)
import CE.Resolve.Request
import Data.Aeson (Value (..), decodeStrict, encode, toJSON)
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as BL
import ReferenceResolveGen
import WireHarness (field, refusedBy, runLegs, setKey)

battery :: IO Bool
battery = runLegs names probes
 where
  names =
    [ "resolve: the numbering of the segment table moves no answer"
    , "resolve: reversing the sites reverses the answers"
    , "resolve: an unwalked file is never a target"
    , "resolve: counts name the sites, the files and the answered"
    , "resolve: no compile database, no forced arcs"
    , "resolve: a vocabulary of another length is refused by name"
    , "resolve: a file row without its walked column is refused by name"
    , "resolve: a directory named twice is refused by name"
    , "resolve: a site in a language this family does not hold is refused"
    , "resolve: a dotted name with a leading separator is refused"
    , "resolve: a Lua module name that does not alternate is refused"
    , "resolve: the cap counts every table together"
    ]
  probes =
    [ all (\c -> reply (request False c) == reply (request True c)) cases
    , all reversible (take 40 cases)
    , unwalkedIsNoTarget
    , all counted (take 40 cases)
    , all (\c -> (reply' (request False c) >>= (`field` "forced")) == Just (toJSON ([] :: [Value]))) (take 20 cases)
    , refused (setKey "vocab" (toJSON [0 :: Int]) base) "vocab: "
    , refused (setKey "files" (toJSON [[0, 0, langPy] :: [Integer]]) base) "file 0: malformed file"
    , refused (setKey "dirs" (toJSON [[0, 0], [0, 0] :: [Integer]]) bare) "a path named twice"
    , refused (setKey "sites" (toJSON [[3, 0, 0, 0, 0] :: [Integer]]) base) "language this family does not resolve"
    , refused (setKey "sites" (toJSON [[langPy, 0, 0, 0, -1, 0] :: [Integer]]) base) "malformed dotted name"
    , refused (setKey "sites" (toJSON [[langLua, kindRequire, 0, 0, 0, 0]]) base) "malformed module name"
    , capLeg
    ]
  base = request False firstCase
  bare = setKey "files" (toJSON ([] :: [Value])) (setKey "sites" (toJSON ([] :: [Value])) base)
  refused r want = refusedBy respond r want

firstCase :: Case
firstCase = case cases of
  (c : _) -> c
  [] -> error "no seeded case"

reply :: Value -> Maybe Value
reply r = either (const Nothing) decodeStrict (respond "8.1.0" (BL.toStrict (encode r)))

reply' :: Value -> Maybe (KM.KeyMap Value)
reply' r = case reply r of
  Just (Object o) -> Just o
  _ -> Nothing

-- | Reversed sites, reversed answers.
reversible :: Case -> Bool
reversible c =
  let (r, fs, ds) = lowered c
      (r', _, _) = lowered c {cSites = reverse (cSites c)}
   in fmap reverse (reply r >>= answers fs ds) == (reply r' >>= answers fs ds)

-- | A Python site whose only match is a file the walk did not hold.
unwalkedIsNoTarget :: Bool
unwalkedIsNoTarget =
  let c walkedM = firstCase {cFiles = ["a/n.py"] <> ["a/m.py" | walkedM], cExtra = ["a/m.py" | not walkedM], cSites = [(langPy, 0, "a/n.py", ".m")]}
      ask x = let (r, fs, ds) = lowered x in reply r >>= answers fs ds
   in ask (c True) == Just [RFile "a/m.py" 1] && ask (c False) == Just [RUnres OutOfScope]

-- | counts.sites, counts.files and counts.resolved agree with the rows.
counted :: Case -> Bool
counted c =
  let (r, fs, ds) = lowered c
      rows = reply r >>= answers fs ds
      counts = reply' r >>= (`field` "counts")
      answered = length . filter (\a -> case a of RUnres _ -> False; _ -> True)
   in case (rows, counts) of
        (Just as, Just (Object o)) ->
          KM.lookup "sites" o == Just (toJSON (length (cSites c)))
            && KM.lookup "files" o == Just (toJSON (length fs))
            && KM.lookup "resolved" o == Just (toJSON (answered as))
        _ -> False

-- | One row past the cap degrades; at the cap it does not.
capLeg :: Bool
capLeg = overCap (req (resolveCap + 1)) && not (overCap (req resolveCap))
 where
  req n = ResolveReq Null 1 [] [] (replicate (fromInteger n) [0, 0]) [] [] (PyFacts [] []) (LuaFacts [] []) (GoFacts [] []) (CFacts [] [] [] [] [] [] [] [])
