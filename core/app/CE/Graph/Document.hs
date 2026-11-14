-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce deadcode` document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5; assembled on the measuring
-- side until cacc2741, the reason words until e35fde53;
-- cli/src/graph/deadcode.rs and cli/src/graph/deadcode/advisory.rs
-- still gather what it states): the file-tier
-- dead verdicts with their reason and trust, the aggregates reported
-- and never called dead, the counts, the degraded reason, and — when
-- the road asked for it — the symbol advisory, one row per name the
-- measuring side's table holds for each row the core kept. The graph
-- screen embeds this same document (CE.Graph.Screen), so its
-- statement and assembly are exported.
module CE.Graph.Document (advisoryNames, doc, statement, verdictName, whyCodes) where

import CE.Document.Contract
import CE.Graph.Cost (blockStates, rootStates)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import Data.Bits (testBit)
import Data.Aeson.Types (Pair)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)
import qualified Data.Set as S

-- | The catalogue lists the degraded reasons the reply is sent by
-- (plan v2.32 step 4B).
doc :: DocFamily
doc = docFamily "deadcode" schemaId statement checked deadcode ["reasons" .= coreReasons]

schemaId :: String
schemaId = "ce.deadcode-report/0.5.0"

-- | The request: `nodes` the wire's dense assignment (every reference
-- is a node), a dead row graph/1's [node, verdict] or [node, verdict,
-- trust], a reported row [node, verdict], an advisory row [k, node,
-- line, code] — one per name, `k` its place — and the three advisory
-- bits: whether the road asked, whether the core dropped the table,
-- whether the producer cut the candidates. `files` (the file nodes)
-- and `check` (the face's `--check`) are the console lines' (step 5,
-- CE.Graph.Lines). The grouping (plan v2.35, the dead-file
-- amendment, 5A) is optional and additive: `roots` [node] the roots the
-- graph reply named, `under` [dependent, root] its pairs, `underDropped`
-- the core's marker when it dropped them; absent, every row and count
-- is what it was before the grouping. The gate (5B) is optional and
-- additive too: `states` [node, state] every dead row's state code
-- (CE.Graph.Cost.rootStates; roots and dependents alike, one row each),
-- and `blocked` [node] the files the graph answered as blocking (the
-- roots of the dead files in a blocked state), beside the facts
-- `blockMask` (the project's mask over Cost.blockStates), `compared` (a
-- comparison point rode) and `unanswered` (the reply's
-- `blockUnanswered`).
statement :: String
statement =
  "range nodes\nrange advisory\nrange why\n\
  \fact unresolvedSites judged\nfact asked judged\nfact dropped judged\nfact cut judged\nfact files judged\nfact check kept\noptional files check\n\
  \rows kept 1 judged -\nrows reason 1 judged -\n\
  \rows dead 2+ judged nodes -\nrows reported 2 judged nodes -\n\
  \rows unmentioned 4 judged advisory nodes - -\n\
  \rows roots 1 judged nodes\nrows under 2 judged nodes nodes\nfact underDropped judged\noptional roots under underDropped\n\
  \rows states 2 judged nodes -\nrows blocked 1 judged nodes\nfact blockMask judged\nfact compared judged\nfact unanswered judged\noptional states blocked blockMask compared unanswered\n\
  \ref path nodes\nref node_name nodes\nref symbol advisory\nref why why\n"

-- | The four verdicts by code 1..4 (CE.Graph.Dead).
verdictNames :: [String]
verdictNames = words "unref_private unref_public unreach_private unreach_public"

verdictName :: Integer -> Value
verdictName v = spelled verdictNames (v - 1)

-- | The liveness reason by code: 0 the unreferenced verdicts, 1 the
-- unreachable ones (the English word every machine face prints).
whyCodes :: [String]
whyCodes = ["no kept in-edge and no entry flag", "referenced only from unreferenced files; no entry flag"]

-- | The advisory's codes (CE.Graph.Advisory.code) and the reading of
-- each: how far the declaration's own package lets it out.
advisoryNames, advisoryWhy :: [String]
advisoryNames = words "public_unmentioned private_unmentioned restricted_unmentioned reexported_unmentioned"
advisoryWhy =
  [ "no other file spells this exported name"
  , "no other file spells it; reachable only inside its own package"
  , "no other file spells it; visible to its crate alone"
  , "no other file spells it; a façade re-exports it"
  ]

checked :: DocReq -> Maybe String
checked req =
  asum
    [ asum [single req t | t <- ["kept", "reason"]]
    , codes req "reason" 0 0 (toInteger (length coreReasons) - 1)
    , codes req "dead" 1 1 4
    , codes req "dead" 2 0 2
    , asum [Just ("dead " <> show i <> ": more than three columns") | (i, r) <- zip [0 :: Int ..] (rows req "dead"), length r > 3]
    , codes req "reported" 1 1 4
    , dense req "unmentioned" (range req "advisory")
    , codes req "unmentioned" 3 0 3
    , bits req ["asked", "dropped", "cut"]
    , if flag req "asked" || not (flag req "dropped" || flag req "cut") then Nothing else Just "advisory: bits without the road"
    , if (flag req "asked" && not (flag req "dropped")) || null (rows req "unmentioned") then Nothing else Just "unmentioned: rows the road did not keep"
    , grouping req
    , gated req
    ]

-- | Whether the grouping rode: the `roots` table is in the request.
told :: DocReq -> Bool
told req = maybe False (M.member "roots") (dRows req)

-- | The grouping held to the dead table: every root a dead row, every
-- pair a dependent (a dead row that is no root) beside a root, no pair
-- beside the dropped marker, and, unless dropped, no dependent without
-- a pair.
grouping :: DocReq -> Maybe String
grouping req =
  asum
    [ bits req ["underDropped"]
    , if told req || not (flag req "underDropped") then Nothing else Just "underDropped: without the roots table"
    , asum [Just ("roots " <> show i <> ": not a dead row") | (i, n : _) <- zip [0 :: Int ..] (rows req "roots"), S.notMember n deads]
    , asum [Just ("under " <> show i <> ": not a dependent beside a root") | (i, d : r : _) <- zip [0 :: Int ..] (rows req "under"), S.notMember d deads || S.member d roots || S.notMember r roots]
    , if flag req "underDropped" && not (null (rows req "under")) then Just "under: rows beside the dropped marker" else Nothing
    , if told req && not (flag req "underDropped") && not (S.null (S.difference (S.difference deads roots) covered)) then Just "under: a dependent without a root" else Nothing
    ]
 where
  deads = S.fromList [n | n : _ <- rows req "dead"]
  roots = S.fromList [n | n : _ <- rows req "roots"]
  covered = S.fromList [d | d : _ <- rows req "under"]

-- | Whether the states rode (the `states` table is in the request), and
-- whether the gate rode (the `blocked` table is).
stateful, gateRode :: DocReq -> Bool
stateful req = maybe False (M.member "states") (dRows req)
gateRode req = maybe False (M.member "blocked") (dRows req)

-- | The gate held to the dead table: each state a code, one row per dead
-- row in its order; the two facts bits, the mask over the blockable
-- states; every blocked node a dead row whose state the mask blocks (the
-- core's law blocks the roots of that set; the document holds the
-- membership); nothing blocked when the gate went unanswered.
gated :: DocReq -> Maybe String
gated req =
  asum
    [ codes req "states" 1 0 (toInteger (length rootStates) - 1)
    , if stateful req && [n | n : _ <- rows req "states"] /= [n | n : _ <- rows req "dead"] then Just "states: not one row per dead row, in its order" else Nothing
    , bits req ["compared", "unanswered"]
    , if fact req "blockMask" < 2 ^ length blockStates then Nothing else Just "facts: blockMask outside the blockable states"
    , if gateRode req && not (told req && stateful req) then Just "blocked: without the roots and states tables" else Nothing
    , asum [Just ("blocked " <> show i <> ": not an unreferenced file the mask blocks") | (i, n : _) <- zip [0 :: Int ..] (rows req "blocked"), not (blocks req sts n)]
    , if flag req "unanswered" && not (null (rows req "blocked")) then Just "blocked: rows beside an unanswered gate" else Nothing
    ]
 where
  sts = stateMap req

-- | Every dead file's state code, built once per document (a lookup per
-- row would be quadratic); a node the map lacks reads 0, untold.
stateMap :: DocReq -> M.Map Integer Integer
stateMap req = M.fromList [(r, s) | [r, s] <- rows req "states"]

stateIn :: M.Map Integer Integer -> Integer -> Integer
stateIn m n = M.findWithDefault 0 n m

-- | Whether the mask blocks a dead file, by its state.
blocks :: DocReq -> M.Map Integer Integer -> Integer -> Bool
blocks req m n = case lookup (nameOf rootStates (stateIn m n)) (zip blockStates [0 ..]) of
  Just b -> testBit (fact req "blockMask") b
  Nothing -> False

-- | The document; the advisory's three keys only when the road asked.
deadcode :: DocReq -> Value
deadcode req =
  object $
    [ "dead" .= [gateRow req (sts, blocked) n (dead g row) | row@(n : _) <- rows req "dead"]
    , "reported" .= [object ["name" .= ref "node_name" [n], "verdict" .= verdictName v] | [n, v] <- rows req "reported"]
    , "counts" .= object (["nodes" .= range req "nodes", "kept_edges" .= fromMaybe 0 (keptOf req)] <> groupCounts req <> stateCounts req)
    , "unresolved_sites" .= fact req "unresolvedSites"
    , "degraded" .= degradedOf req ["reason"]
    ]
      -- the advisory's keys only when the road asked; the grouping's
      -- and the gate's ride beside them either way (parenthesised: an
      -- unbracketed `else []` swallowed every key after it)
      <> ( if flag req "asked"
            then
              [ "unmentioned" .= map advisory (rows req "unmentioned")
              , "unmentioned_dropped" .= flag req "dropped"
              , "unmentioned_cut" .= (flag req "cut" && not (flag req "dropped"))
              ]
            else []
         )
      <> ["under_dropped" .= True | told req, flag req "underDropped"]
      <> ["gate" .= gateObj req | gateRode req]
 where
  -- built once per document, read per row
  g = grouped req
  sts = stateMap req
  blocked = S.fromList [n | n : _ <- rows req "blocked"]

-- | The gate as the project declared and the graph answered it: the
-- blocked states by name, what the states were compared against, how
-- many files block, and whether the gate could be answered at all.
gateObj :: DocReq -> Value
gateObj req =
  object
    [ "block" .= [s | (b, s) <- zip [0 ..] blockStates, testBit (fact req "blockMask") b]
    , "comparison" .= (if flag req "compared" then toJSON ("baseline" :: String) else Null)
    , "blocked" .= length (rows req "blocked")
    , "answered" .= not (flag req "unanswered")
    ]

-- | A dead file's row under the gate, root or dependent: its state by
-- name and, when the gate rode, whether it blocks; every row of a
-- request without states as it was.
gateRow :: DocReq -> (M.Map Integer Integer, S.Set Integer) -> Integer -> Value -> Value
gateRow req (sts, blocked) n v = case v of
  Object o
    | stateful req ->
        Object (o <> KM.fromList (["state" .= nameOf rootStates (stateIn sts n)] <> ["blocking" .= S.member n blocked | gateRode req]))
  _ -> v

-- | One count per state over the unreferenced files, when the states
-- rode.
stateCounts :: DocReq -> [Pair]
stateCounts req
  | stateful req = [Key.fromString s .= length [() | [_, st] <- rows req "states", st == c] | (c, s) <- zip [0 ..] rootStates]
  | otherwise = []

-- | The grouping as the rows read it: the roots, and each dependent's
-- roots ascending (the pairs ride ascending); Nothing when it did not
-- ride.
grouped :: DocReq -> Maybe (S.Set Integer, M.Map Integer [Integer])
grouped req
  | told req = Just (S.fromList [n | n : _ <- rows req "roots"], M.fromListWith (flip (<>)) [(d, [r]) | d : r : _ <- rows req "under"])
  | otherwise = Nothing

-- | The counts the grouping adds: roots and dependents.
groupCounts :: DocReq -> [Pair]
groupCounts req = case grouped req of
  Just (roots, _) -> ["roots" .= S.size roots, "dependents" .= (length (rows req "dead") - S.size roots)]
  Nothing -> []

keptOf :: DocReq -> Maybe Integer
keptOf req = case rows req "kept" of
  [k : _] -> Just k
  _ -> Nothing

-- | One dead file: its verdict, the reason by the verdict's family,
-- and its trust (null on a two-column row); under the grouping, whether
-- it is a root and, a dependent, its roots' names ascending.
dead :: Maybe (S.Set Integer, M.Map Integer [Integer]) -> [Integer] -> Value
dead g row = case row of
  n : v : trust ->
    let code = if v > 2 then 1 else 0 :: Integer
     in object $
          [ "name" .= ref "path" [n]
          , "verdict" .= verdictName v
          , "why" .= spelled whyCodes code
          , "whyCode" .= code
          , "confidence" .= maybe Null toJSON (headOf trust)
          ]
            <> case g of
              Nothing -> []
              Just (roots, under)
                | S.member n roots -> ["root" .= True]
                -- the pairs' order, i.e. node order; the measuring side
                -- assigns nodes by path, so the names read ascending
                | otherwise -> ["root" .= False, "under" .= [ref "path" [r] | r <- M.findWithDefault [] n under]]
  _ -> Null
 where
  headOf xs = case xs of
    x : _ -> Just x
    [] -> Nothing

-- | One advisory row: the declaring file, the name, its line, the
-- code by name and its reading.
advisory :: [Integer] -> Value
advisory row = case row of
  [k, n, line, code] ->
    object
      [ "name" .= ref "path" [n]
      , "symbol" .= ref "symbol" [k]
      , "line" .= line
      , "code" .= spelled advisoryNames code
      , "why" .= spelled advisoryWhy code
      ]
  _ -> Null
