-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce merge` document (plan v2.32 step 3; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/merge/face.rs):
-- every judged group with its family, its members, the parameters in
-- the order its holes first reach them — each with every member's
-- text at that parameter's first hole — the member kept, the lines
-- saved and the reason; the groups not sent, counted by why; the
-- counts. The measuring side sends every chunk's answer already
-- joined: the suggestion rows numbered across chunks, one row per
-- member numbered across groups (the member number every reference
-- carries), the hole rows, and the facts only it knows.
module CE.Merge.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = docFamily "merge" schemaId statement joined assemble []

schemaId :: String
schemaId = "ce.merge-report/0.1.0"

-- | A group row is merge/1's suggestion [g, params, kept, savings,
-- feasible, reason] with the family and fragment bits after it; a
-- member row [k, g, m, first line, last line, run first, run last,
-- has a unit]; a hole row merge/1's [g, hole, param, m, post,
-- postEnd]. A degraded document carries nothing, the measuring side's
-- counts included.
-- References: path [member], unit [member], text [member, post, postEnd], why [text].
statement :: String
statement =
  "range members\nrange why\n\
  \fact nodes judged\nfact merged_duplicates judged\n\
  \fact not_isomorphic judged\nfact no_slot_table judged\nfact unbuilt judged\nfact over_cap judged\n\
  \rows groups 8 judged - - - - - - - -\n\
  \rows members 8 judged members - - - - - - -\n\
  \rows holes 6 judged - - - - - -\n\
  \ref path members\nref unit members\nref text members - -\nref why why\n"

-- | The reasons by code (CE.Merge.Cost): feasible, a position no
-- parameter can stand for, a type position, a gap across statements,
-- more parameters than the ceiling, no line saved.
reasons :: [String]
reasons = ["ok", "position", "type", "spans_statements", "too_many_params", "no_savings"]

-- | Groups numbered in order, members numbered in order and grouped
-- in group order with their m from 0, every reason known, every hole
-- on a member of its group.
joined :: DocReq -> Maybe String
joined req =
  asum
    [ dense req "groups" (toInteger (length (rows req "groups")))
    , dense req "members" (range req "members")
    , if map (take 2 . drop 1) (rows req "members") == expected then Nothing else Just "members: not grouped in order"
    , codes req "groups" 5 0 (toInteger (length reasons) - 1)
    , asum [Just ("holes " <> show i <> ": no such member") | (i, g : _ : _ : m : _) <- zip [0 :: Int ..] (rows req "holes"), M.notMember (g, m) ids]
    ]
 where
  ids = memberIds req
  sizes = M.fromListWith (+) [(g, 1 :: Integer) | _ : g : _ <- rows req "members"]
  expected = [[g, m] | (g, n) <- M.toAscList sizes, m <- [0 .. n - 1]]

-- | (group, m) → the member's number.
memberIds :: DocReq -> M.Map (Integer, Integer) Integer
memberIds req = M.fromList [((g, m), k) | k : g : m : _ <- rows req "members"]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= schemaId
    , "counts" .= counted countKeys tallies
    , "unsendable" .= counted unsendable (map (fact req) unsendable)
    , "groups" .= map (group ids (bucket 1 "members") (bucket 0 "holes")) (rows req "groups")
    , "degraded" .= whyRef req
    ]
 where
  ids = memberIds req
  bucket col t = M.fromListWith (flip (<>)) [(g, [r]) | r <- rows req t, g <- take 1 (drop col r)]
  countKeys = words "groups members nodes suggestions holes feasible merged_duplicates"
  unsendable = words "not_isomorphic no_slot_table unbuilt over_cap"
  feasible = toInteger (length [() | r <- rows req "groups", take 1 (drop 4 r) == [1]])
  size = toInteger . length . rows req
  tallies = [size "groups", size "members", fact req "nodes", size "groups", size "holes", feasible, fact req "merged_duplicates"]

-- | One group, its member and hole rows bucketed by group: its
-- members, and its parameters in first-reach order, each with every
-- member's text at the parameter's first hole.
group :: M.Map (Integer, Integer) Integer -> M.Map Integer [[Integer]] -> M.Map Integer [[Integer]] -> [Integer] -> Value
group ids members holes row = case row of
  [g, params, kept, savings, feasible, reason, fam, fragment] ->
    let mine = M.findWithDefault [] g holes
        firstReach = foldl reach [] [(p, h) | _ : h : p : _ <- mine]
        reach acc (p, h) = if p `elem` map fst acc then acc else acc <> [(p, h)]
        value r = case r of
          [_, _, _, m, post, postEnd] -> object ["member" .= m, "text" .= ref "text" [M.findWithDefault (-1) (g, m) ids, post, postEnd]]
          _ -> Null
     in object
          [ "group" .= g
          , "family" .= (if fam == 0 then "t1t2" else "t3" :: String)
          , "fragment" .= (fragment /= 0)
          , "members" .= map member (M.findWithDefault [] g members)
          , "params" .= params
          , "kept" .= kept
          , "savings" .= savings
          , "feasible" .= (feasible == 1)
          , "reason" .= concat (take 1 (drop (fromInteger reason) reasons))
          , "holes" .= [object ["param" .= p, "values" .= [value r | r <- mine, take 1 (drop 1 r) == [h]]] | (p, h) <- firstReach]
          ]
  _ -> Null

member :: [Integer] -> Value
member row = case row of
  [k, _, _, a, b, ra, rb, unit] ->
    object
      [ "path" .= ref "path" [k]
      , "unit" .= (if unit /= 0 then ref "unit" [k] else Null)
      , "lines" .= [a, b]
      , "run" .= [ra, rb]
      ]
  _ -> Null
