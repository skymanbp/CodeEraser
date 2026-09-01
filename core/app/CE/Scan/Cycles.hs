-- | The recursion increment (S3776 §1, ADR-008 fourth instalment):
-- every unit sitting in a call cycle pays one flat point, direct or
-- indirect. The whitepaper says `each method in a recursion cycle,
-- whether direct or indirect` (v1.7 p.8 and Appendix B1), and its
-- change log v1.1 widened the rule to the indirect case on purpose —
-- so a cycle, not a self-call, is what has to be found.
--
-- This is a JUDGMENT and therefore lives here rather than reusing
-- CE.Graph.Cycles: that module declares its own stance in its first
-- line — cycles are REPORTED, never judged (RG9) — and the two must
-- not be confused for sharing an idiom. What IS reused, deliberately
-- and verbatim, is its reading of a singleton (Graph/Cycles.hs): one
-- vertex is a cycle exactly through its own arc, so direct recursion
-- needs no special case at all — it is a cycle of length one.
--
-- The arcs arrive as indices into the request's own rows array, and
-- both ends must be cognitive rows. No name, no path and no subject
-- crosses (§5.9.2); the client proves the arcs are intra-file by
-- construction, which is why nothing here can — or tries to — check
-- it. The increment itself, the constant 1, exists in this one place.
module CE.Scan.Cycles (callBattery, withCycles) where

import Control.Applicative ((<|>))
import Data.Foldable (asum)
import qualified Data.Graph as G
import qualified Data.IntSet as IS
import Data.List (find)
import qualified Data.Set as S
import Data.Tree (flatten)

-- | The cognitive metric's frozen code — the row position an arc
-- endpoint must land on (CE.Scan.Cost's table names it 4).
cognitiveCode :: Integer
cognitiveCode = 4

-- | The call table's own boundary contract, in request order like
-- every other battery here: each row is a pair, both ends index a
-- row that exists AND is a cognitive row, and the table ascends
-- strictly — the order that makes an arc list an arc set, so the
-- same graph can never arrive twice spelled differently.
callBattery :: [[Integer]] -> Maybe [[Integer]] -> Maybe String
callBattery _ Nothing = Nothing
callBattery rows (Just arcs) =
  asum (zipWith shape [0 :: Int ..] arcs) <|> ascending
 where
  n = toInteger (length rows)
  cog = IS.fromList [i | (i, code : _) <- zip [0 ..] rows, code == cognitiveCode]
  seats end = end >= 0 && end < n && IS.member (fromInteger end) cog
  shape i row = case row of
    [from, to]
      | from < 0 || from >= n || to < 0 || to >= n ->
          Just (label i <> "endpoint outside the rows")
      | not (seats from) || not (seats to) ->
          Just (label i <> "endpoint is not a cognitive row")
      | otherwise -> Nothing
    _ -> Just (label i <> "malformed row (need [from,to])")
  ascending = out <$> find (uncurry (>=)) (zip arcs (drop 1 arcs))
  out _ = "callEdges: not strictly ascending"
  label i = "callEdge " <> show i <> ": "

-- | The rows as judged, beside the table of what moved: every
-- cognitive row whose unit sits in a cycle carries value + 1, and
-- the same rows are echoed as [rowIndex, effectiveValue] so the
-- measuring side can render the number the judgment used without
-- ever deriving the cycle — or the increment — for itself. Absent
-- table = a client that sent no arcs: the rows and the reply keep
-- their bytes.
withCycles :: Maybe [[Integer]] -> [[Integer]] -> ([[Integer]], [[Integer]])
withCycles Nothing rows = (rows, [])
withCycles (Just arcs) rows = (zipWith bump [0 ..] rows, moved)
 where
  n = length rows
  pairs = [(fromInteger a, fromInteger b) | [a, b] <- arcs]
  arcSet = S.fromList pairs
  looped = IS.fromList (if n <= 0 then [] else concatMap members (G.scc (G.buildG (0, n - 1) pairs)))
  members t = let vs = flatten t in if cyclic vs then vs else []
  cyclic [v] = S.member (v, v) arcSet
  cyclic _ = True
  raised i row = case row of
    [code, v] | code == cognitiveCode, IS.member i looped -> Just (v + 1)
    _ -> Nothing
  bump i row = maybe row (\v -> [cognitiveCode, v]) (raised i row)
  moved = [[toInteger i, v] | (i, row) <- zip [0 ..] rows, Just v <- [raised i row]]
