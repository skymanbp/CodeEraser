{-# LANGUAGE OverloadedStrings #-}

-- | scan/1's complexity derivation held to ReferenceScan, a second
-- spelling that reads structure trees instead of events: twelve
-- hundred seeded programs in three table families, each sent through
-- the REAL respond, must echo the reference's numbers row for row —
-- cyclomatic, cognitive after the recursion increment, depth — and
-- the raised rows; the whitepaper register's thirty-eight lines, the
-- Rust emitter's events for each port, must derive what the page's
-- tree measures; three hundred dense call graphs must raise exactly
-- the units the reference's closure puts on a cycle.
module ScanReferenceProps (battery) where

import CE.Scan (respond)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import Data.Maybe (mapMaybe)
import qualified Data.Set as S
import ReferenceScan (cyclic, measure, settled)
import ReferenceScanGen
import ScanPageTrees (pageTree)
import WireHarness (field, replyObjWith, runChecks, setKey)
import qualified WireHarness as H

battery :: IO Bool
battery = do
  raw <- B8.readFile "../contracts/fixtures/scan/whitepaper.ndjson"
  let register = mapMaybe decodeStrict (filter (not . B8.null) (map stripCR (B8.lines raw)))
  runChecks
    ( H.battery ("v2.33: 1200 seeded programs echo the tree reference's derived and cocBumped rows", "v2.33: the programs reach every family, structure and cycle shape") (map judged programs) shapes (S.unions (map reached programs))
        <> H.battery ("v2.33: 300 seeded call graphs raise exactly the closure's cycle", "v2.33: the graphs reach a self-loop, a longer cycle and an acyclic graph") (map judged callGraphs) graphShapes (S.unions (map graphReached callGraphs))
        <> [ ("v2.33: every whitepaper line derives on the wire what its page tree measures", length register >= 38 && all pageAgrees register)
           , ("v2.33: the recursion anchor settles to the page's value plus one", any anchorSettles register)
           ]
    )
 where
  stripCR l = if not (B8.null l) && B8.last l == '\r' then B8.init l else l

-- | The reply's two echoes beside the reference's; Nothing = agreed.
judged :: Program -> Maybe String
judged p
  | got == Just want = Nothing
  | otherwise = Just (take 300 (show got <> " /= " <> show want))
 where
  (derived, bumped) = settled p
  want = (toJSON derived, toJSON bumped)
  got = do
    o <- replyObjWith respond (request p)
    (,) <$> field o "derived" <*> field o "cocBumped"

-- | One register line, the columns read here.
data Line = Line {lName, lExt :: String, lEvents :: [[Integer]], lCoc :: Integer, lCc :: Maybe Integer, lSettled :: Maybe Integer}

instance FromJSON Line where
  parseJSON = withObject "line" $ \o -> do
    want <- o .: "want"
    settledCoc <- o .:? "settled" >>= traverse (.: "coc")
    Line <$> o .: "name" <*> o .: "ext" <*> o .: "events" <*> want .: "coc" <*> want .:? "cc" <*> pure settledCoc

-- | The line's events through the wire, one unit at rows 0–2; looping
-- adds the unit's call to itself and reads the raised row instead.
lineReply :: Line -> Bool -> Maybe Value
lineReply l looping = replyObjWith respond req >>= (`field` key)
 where
  base = setKey "events" (toJSON (map (1 :) (lEvents l))) (H.rowsRequest "7.0.0" "scan.request" [[3, 0], [4, 0], [5, 0]])
  req = if looping then setKey "callEdges" (toJSON [[1, 1 :: Integer]]) base else base
  key = if looping then "cocBumped" else "derived"

pageAgrees :: Line -> Bool
pageAgrees l = case pageTree (lName l) (lExt l) of
  Nothing -> False
  Just tree ->
    let (cc, coc, depth) = measure tree
     in lineReply l False == Just (toJSON [[0, cc], [1, coc], [2, depth]])
          && coc == lCoc l
          && maybe True (== cc) (lCc l)

anchorSettles :: Line -> Bool
anchorSettles l = case (lSettled l, pageTree (lName l) (lExt l)) of
  (Just want, Just tree) ->
    let (_, coc, _) = measure tree
     in S.member 0 (cyclic [(0, 0)]) && coc + 1 == want && lineReply l True == Just (toJSON [[1, want]])
  _ -> False

shapes :: [String]
shapes = ["CLike", "GoLike", "PyLike", "else-if", "bare else", "loop else", "let chain", "??", "lambda", "ternary", "labelled jump", "nested unit", "non-decision case", "mixed chain", "catch", "cycle"]

reached :: Program -> S.Set String
reached p =
  S.fromList ([show (pStyle p)] <> ["nested unit" | length (units p) > length (pTop p)] <> ["cycle" | not (S.null (cyclic (pArcs p)))])
    <> foldMap (foldMap stmtTags) (units p)

stmtTags :: Stmt -> S.Set String
stmtTags s = case s of
  Do e -> exprTags e
  If c b t -> exprTags c <> foldMap stmtTags b <> tailTags t
  Loop h b e -> exprTags h <> foldMap stmtTags b <> maybe S.empty (\x -> S.insert "loop else" (foldMap stmtTags x)) e
  Switch v cs -> exprTags v <> mconcat [S.fromList ["non-decision case" | not d] <> foldMap stmtTags b | Case d b <- cs]
  Try b cs f -> S.fromList ["catch" | not (null cs)] <> foldMap stmtTags (b <> concat cs <> f)
  Jump True -> S.singleton "labelled jump"
  Block b -> foldMap stmtTags b
  _ -> S.empty
 where
  tailTags t = case t of
    NoElse -> S.empty
    Else braced b -> S.fromList ["bare else" | not braced] <> foldMap stmtTags b
    ElseIf c b t' -> S.insert "else-if" (exprTags c <> foldMap stmtTags b <> tailTags t')

exprTags :: Expr -> S.Set String
exprTags e = case e of
  Bin Coalesce l r -> S.insert "??" (exprTags l <> exprTags r)
  Bin o l r -> S.fromList ["mixed chain" | any (/= o) (ops l <> ops r)] <> exprTags l <> exprTags r
  Paren x -> exprTags x
  LetChain xs -> S.insert "let chain" (foldMap exprTags xs)
  Lambda b -> S.insert "lambda" (foldMap stmtTags b)
  Ternary c a b -> S.insert "ternary" (foldMap exprTags [c, a, b])
  Leaf -> S.empty
 where
  ops (Bin o l r) | o /= Coalesce = o : ops l <> ops r
  ops _ = []

graphShapes :: [String]
graphShapes = ["self-loop", "longer cycle", "acyclic"]

graphReached :: Program -> S.Set String
graphReached p =
  S.fromList (["self-loop" | any (uncurry (==)) arcs] <> ["longer cycle" | any (`S.notMember` selfs) (S.toList looped)] <> ["acyclic" | S.null looped])
 where
  arcs = pArcs p
  looped = cyclic arcs
  selfs = S.fromList [a | (a, b) <- arcs, a == b]
