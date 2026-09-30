-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The flow battery's hand-written cases (plan v2.31 step 3) as text
-- tables: a judgment case is a header line `# <leg name>`, its rows
-- (`u` unit, `s` statement, `v` variable, `x` access) and the finding
-- rows it expects (`=`; none = no finding); a refusal case (module
-- FlowRefusals) is a header `! <message>` and the smallest rows that
-- trip it. A case without a `u` row gets unit 0, lang 3 and the
-- parameter count its `v` rows show. Text, not lists of row literals:
-- those normalise to one token run and the clone gate named forty
-- pairs on the first draft; each literal stays under the function
-- wall in turn.
module FlowCases (Case (..), casesOf, judgments) where

import Data.Aeson (Value)
import WireHarness (tabledRequest)

data Case = Case {caseName :: String, caseRequest :: Value, caseExpect :: [[Integer]], caseRefusal :: Maybe String}

judgments :: [Case]
judgments = casesOf (judgmentsA <> judgmentsB <> judgmentsC)

-- | A header line opens a case; the lines up to the next header are
-- its rows.
casesOf :: String -> [Case]
casesOf text = go (lines text)
 where
  go [] = []
  go (header : rest) = let (body, next) = break isHeader rest in caseOf header body : go next
  isHeader l = take 1 l `elem` ["#", "!"]

caseOf :: String -> [String] -> Case
caseOf header body = Case name request (table '=') refusal
 where
  name = drop 2 header
  refusal = if take 1 header == "!" then Just name else Nothing
  request = tabledRequest "7.0.0" "flow.request" [("units", units), ("stmts", table 's'), ("vars", table 'v'), ("uses", table 'x')]
  table :: Char -> [[Integer]]
  table tag = [map read (words rest) | (t : rest) <- body, t == tag]
  units = case table 'u' of
    [] -> [[0, 3, toInteger (length [() | [_, _, -1, _] <- table 'v'])]]
    rows -> rows

-- | Kinds: 0 block 1 stmt 2 if 3 loop 4 switch 5 case 6 try 7 catch
-- 8 finally 9 return 10 throw 11 break 12 continue 13 goto 14 label
-- 15 noreturn; stmt flags 1 else 2 infinite 4 fallthrough 8 dynamic;
-- var flags 1 param 2 captured 4 ignored 8 address; modes 0 read
-- 1 write 2 readwrite; findings 0 unreachable 1 dead store 2 unused
-- local 3 unused param. The findings kinds, one each:
judgmentsA :: String
judgmentsA =
  "# an unreachable run after a return is one finding, seq to seqEnd, and covers a whole subtree\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 1 -1 9 0 0\n\
  \s 0 2 -1 1 0 0\n\
  \s 0 3 -1 2 1 0\n\
  \s 0 4 3 1 0 0\n\
  \s 0 5 3 1 0 0\n\
  \= 0 0 2 -1 5\n\
  \# a dead store is a write no path reads before the next write\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 1 -1 1 0 0\n\
  \s 0 2 -1 1 0 0\n\
  \v 0 0 0 0\n\
  \x 0 0 0 1\n\
  \x 0 1 0 1\n\
  \x 0 2 0 0\n\
  \= 0 1 0 0 0\n\
  \# a read on any path keeps a store live\n\
  \s 0 0 -1 1 0 0\n\
  \s 0 1 -1 2 0 0\n\
  \s 0 2 1 1 0 0\n\
  \s 0 3 -1 1 0 0\n\
  \s 0 4 -1 1 0 0\n\
  \v 0 0 0 0\n\
  \x 0 0 0 1\n\
  \x 0 2 0 0\n\
  \x 0 3 0 1\n\
  \x 0 4 0 0\n\
  \# an unused local and an unused parameter are named at their declaration; a variable nothing reads gets no dead-store finding\n\
  \s 0 0 -1 1 0 0\n\
  \v 0 0 -1 1\n\
  \v 0 1 0 0\n\
  \x 0 0 1 1\n\
  \= 0 2 0 1 0\n\
  \= 0 3 -1 0 -1\n"

-- | try / catch / finally, the loops and the switch.
judgmentsB :: String
judgmentsB =
  "# try, catch, finally: every body statement may reach the catch, so the read after the try is reachable and the store before the return live\n\
  \s 0 0 -1 6 0 0\n\
  \s 0 1 0 1 0 0\n\
  \s 0 2 0 9 0 0\n\
  \s 0 3 0 7 0 0\n\
  \s 0 4 3 1 0 0\n\
  \s 0 5 0 8 0 0\n\
  \s 0 6 5 1 0 0\n\
  \s 0 7 -1 1 0 0\n\
  \v 0 0 1 0\n\
  \x 0 1 0 1\n\
  \x 0 6 0 0\n\
  \x 0 7 0 0\n\
  \# a return runs the finally and leaves: without a catch the statement after the try is unreachable and the store before the return dead\n\
  \s 0 0 -1 6 0 0\n\
  \s 0 1 0 1 0 0\n\
  \s 0 2 0 9 0 0\n\
  \s 0 3 0 8 0 0\n\
  \s 0 4 3 1 0 0\n\
  \s 0 5 -1 1 0 0\n\
  \v 0 0 1 0\n\
  \x 0 1 0 1\n\
  \x 0 5 0 0\n\
  \= 0 0 5 -1 5\n\
  \= 0 1 1 0 1\n\
  \# a constant-true loop with a break falls through, one without ends the unit\n\
  \s 0 0 -1 3 2 0\n\
  \s 0 1 0 1 0 0\n\
  \s 0 2 0 2 0 0\n\
  \s 0 3 2 11 0 0\n\
  \s 0 4 -1 1 0 0\n\
  \s 0 5 -1 3 2 0\n\
  \s 0 6 5 1 0 0\n\
  \s 0 7 -1 1 0 0\n\
  \= 0 0 7 -1 7\n\
  \# fallthrough chains the cases, a break leaves the switch, a default skips nothing\n\
  \s 0 0 -1 4 1 0\n\
  \s 0 1 0 5 4 0\n\
  \s 0 2 1 1 0 0\n\
  \s 0 3 0 5 0 0\n\
  \s 0 4 3 1 0 0\n\
  \s 0 5 3 11 0 0\n\
  \s 0 6 -1 1 0 0\n\
  \v 0 0 2 0\n\
  \x 0 2 0 1\n\
  \x 0 4 0 0\n\
  \x 0 6 0 0\n"

-- | continue, goto and label, the exemptions, the dynamic skip (the
-- table's last case, which FlowProps also reads for its counts).
judgmentsC :: String
judgmentsC =
  "# continue resolves to the loop head: the read after it is unreachable and the store before it dead\n\
  \s 0 0 -1 3 0 0\n\
  \s 0 1 0 1 0 0\n\
  \s 0 2 0 12 0 0\n\
  \s 0 3 0 1 0 0\n\
  \v 0 0 1 0\n\
  \x 0 1 0 1\n\
  \x 0 3 0 0\n\
  \= 0 0 3 -1 3\n\
  \= 0 1 1 0 1\n\
  \# goto and label: a forward jump leaves the run it skips unreachable\n\
  \s 0 0 -1 13 0 2\n\
  \s 0 1 -1 1 0 0\n\
  \s 0 2 -1 14 0 0\n\
  \s 0 3 -1 1 0 0\n\
  \= 0 0 1 -1 1\n\
  \# a backward goto keeps a store live\n\
  \s 0 0 -1 14 0 0\n\
  \s 0 1 -1 1 0 0\n\
  \s 0 2 -1 1 0 0\n\
  \s 0 3 -1 13 0 0\n\
  \v 0 0 1 0\n\
  \x 0 1 0 0\n\
  \x 0 2 0 1\n\
  \# captured, address-taken and ignored variables are never judged\n\
  \s 0 0 -1 1 0 0\n\
  \v 0 0 -1 5\n\
  \v 0 1 0 2\n\
  \v 0 2 0 8\n\
  \v 0 3 0 4\n\
  \x 0 0 1 1\n\
  \x 0 0 2 1\n\
  \x 0 0 3 1\n\
  \# a unit any of whose statements is dynamic is skipped whole\n\
  \s 0 0 -1 1 8 0\n\
  \s 0 1 -1 1 0 0\n\
  \v 0 0 1 0\n\
  \x 0 0 0 1\n\
  \x 0 1 0 1\n"
