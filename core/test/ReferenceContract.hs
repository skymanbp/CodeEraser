-- | The vocabulary the step-7 references share (plan v2.32 step 7,
-- authority-track §7): a family's CONTRACT spoken as labelled
-- offender lists, and the one comparison every equivalence battery
-- makes against a family's real respond. The references themselves
-- stay independent of the shipped code — this module imports nothing
-- from core/app — and the clone gate named the per-family copies of
-- these few lines the moment the third reference minted them.
module ReferenceContract (answers, breaks, cells, firstOffender, knobbedContract, knobbedRequest, labelled, pairTable) where

import qualified Data.ByteString.Char8 as B8
import Data.Aeson (Value, toJSON)
import WireHarness (fieldsOf, refusedBy, rowsRequest, setKey)

-- | Each element's offence under its table's label, in table order:
-- "<what> <i>: <why>".
labelled :: String -> (a -> Maybe String) -> [a] -> [String]
labelled what why xs = [what <> " " <> show i <> ": " <> w | (i, x) <- zip [0 :: Int ..] xs, Just w <- [why x]]

-- | Each neighbour pair that breaks the table's order, labelled by the
-- later element's index.
breaks :: String -> String -> (a -> a -> Bool) -> [a] -> [String]
breaks what msg bad xs = [what <> " " <> show i <> ": " <> msg | (i, (a, b)) <- zip [1 :: Int ..] (zip xs (drop 1 xs)), bad a b]

-- | The first named offender of a contract read in request order.
firstOffender :: [String] -> Maybe String
firstOffender offenders = case offenders of
  (w : _) -> Just w
  [] -> Nothing

-- | The knobbed-table contract (trend/2, tombstone/1): each row, then
-- each knob, then the knob codes strictly ascending.
knobbedContract :: ([Integer] -> Maybe String) -> ([Integer] -> Maybe String) -> [[Integer]] -> [[Integer]] -> Maybe String
knobbedContract rowWhy knobWhy rows knobs =
  firstOffender
    ( labelled "row" rowWhy rows
        <> labelled "knob" knobWhy knobs
        <> breaks "knob" "not strictly ascending" (\a b -> take 1 a >= take 1 b) knobs
    )

-- | A knobbed-table request of the given type: rows and knob rows.
knobbedRequest :: String -> [[Integer]] -> [[Integer]] -> Value
knobbedRequest kind rows knobs = setKey "knobs" (toJSON knobs) (rowsRequest "7.0.0" kind rows)

-- | A two-column text table read into its typed cases.
pairTable :: (Read a, Read b) => [String] -> [(a, b)]
pairTable table = [(read a, read b) | [a, b] <- cells (unlines table)]

-- | A reference's answer against the family's real respond: the same
-- reply fields under the given keys, or a refusal whose message
-- carries the reference's offender.
answers ::
  (String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString) ->
  [String] ->
  Value ->
  Either String [Maybe Value] ->
  Bool
answers respond keys request want = case want of
  Right fields -> fieldsOf respond request keys == Just fields
  Left why -> refusedBy respond request why

-- | A text table: one case per line, cells separated by " ; " — the
-- refusal tables are data, read where they are used.
cells :: String -> [[String]]
cells = map (split . words) . lines
 where
  split ws = case break (== ";") ws of
    (cell, _ : rest) -> unwords cell : split rest
    (cell, []) -> [unwords cell]
