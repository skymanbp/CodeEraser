-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | query.request handler (plan v2.31 step 1; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §4):
-- the thirteenth judgment family — the Datalog behind `ce query`,
-- `ce rules`, the MCP tools and the GUI screen. The measuring side
-- lexes the program into a `[kind, value]` token stream, assembles
-- the fact tables the program names from its index (request-local
-- ids, name hashes; no name, path or source text crosses, §5.9.2) and
-- sends both; this side parses, checks, stratifies, evaluates and
-- answers every query's tuples, every assertion's violations, the
-- proof trees, the program predicates' resolved sorts and the
-- program's errors — each error at a token index the measuring side
-- maps back to `line:column`. A capped family: tokens
-- and fact rows are priced before judging, derived tuples while
-- judging, proof nodes never degrade (they stop and count).
module CE.Query (respond) where

import CE.Query.Check (Checked (..), Goal (..), check)
import CE.Query.Contract (QueryReq (..), facts, offence, overCap, tokens)
import CE.Query.Cost (errSyntax, proofCap)
import CE.Query.Eval (Db, Prov, evalProgram, goalAnswers)
import CE.Query.Parse (parseProgram)
import CE.Query.Proof (Node (..), Root (..), proofRows)
import CE.Query.Schema (schemaRows)
import CE.Query.Syntax (Clause (..))
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import qualified Data.Aeson.Key as Key
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S

-- | decode → cap (tokens, fact rows) → contract → judge.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "query" reqId overCap offence (degraded proto) (judged proto)

-- | Everything a reply carries besides the envelope: the five
-- tables, the counts, the degraded bit.
data Answered = Answered
  { ansGoals :: [[Integer]]
  , ansPreds :: [[Integer]]
  , ansRows :: [[Integer]]
  , ansProof :: [[Integer]]
  , ansErrors :: [[Integer]]
  , ansCounts :: [(String, Integer)]
  , ansDegraded :: Bool
  }

-- | The numbers only an evaluation produces: derived tuples, query
-- answers, assertion violations, proof nodes emitted, proof trees
-- left out.
data Tally = Tally {tDerived, tAnswers, tViolations, tNodes, tSkipped :: Integer}

zeroTally :: Tally
zeroTally = Tally 0 0 0 0 0

-- | Parse, check, evaluate, answer — the first failing phase answers
-- with its errors and empty tables; the derived cap answers degraded
-- with the count it reached.
judged :: String -> QueryReq -> B8.ByteString
judged proto req = reply proto req $ case parseProgram (tokens req) of
  Left at -> withErrors req [] [[toInteger at, errSyntax]]
  Right clauses -> case check (fromInteger (preludeOf req)) clauses of
    Left errs -> withErrors req clauses [[toInteger at, code] | (at, code) <- errs]
    Right chk -> case evalProgram chk (factsDb req) of
      Left n -> blank req clauses (Just chk) zeroTally {tDerived = n} True
      Right out -> answered req clauses chk out

-- | Empty tables under the counts the phases reached — the
-- predicate sorts still ride once the checker resolved them (a
-- derived-cap abort keeps them; an erroring program has none).
blank :: QueryReq -> [Clause] -> Maybe Checked -> Tally -> Bool -> Answered
blank req clauses chk tally = Answered [] (maybe [] predRows chk) [] [] [] (countsOf req clauses chk tally)

-- | `[code, sorts…]` per program predicate, ascending by code: the
-- measuring side labels a proof node's arguments by these, the way
-- it labels a goal's columns by `goals`.
predRows :: Checked -> [[Integer]]
predRows chk = [toInteger p : sorts | (p, sorts) <- chkPreds chk]

withErrors :: QueryReq -> [Clause] -> [[Integer]] -> Answered
withErrors req clauses errs = (blank req clauses Nothing zeroTally False) {ansErrors = errs}

-- | The goals' tables and the proof rows under the node budget: a
-- query's proofs only when asked for (`why`), an assertion's always
-- — a violation without its derivation is a bare accusation.
answered :: QueryReq -> [Clause] -> Checked -> (Db, Prov, Integer) -> Answered
answered req clauses chk (db, prov, derived) =
  Answered goalRows (predRows chk) rows proof [] (countsOf req clauses (Just chk) tally) False
 where
  answersOf = [(g, goal, goalAnswers db goal) | (g, goal) <- zip [0 :: Int ..] (chkGoals chk)]
  goalRows = [toInteger g : goalKind goal : goalSorts goal | (g, goal, _) <- answersOf]
  rows = [toInteger g : t | (g, _, tuples) <- answersOf, (t, _) <- tuples]
  roots =
    [ Root g i (Node (toInteger (goalPred goal)) (toInteger (goalClause goal)) t fs)
    | (g, goal, tuples) <- answersOf
    , goalKind goal == 1 || whyOf req
    , (i, (t, fs)) <- zip [0 ..] tuples
    ]
  (proof, skipped, nodes) = proofRows prov proofCap roots
  count k = toInteger (length [() | (_, goal, tuples) <- answersOf, goalKind goal == k, _ <- tuples])
  tally = Tally derived (count 0) (count 1) nodes skipped

-- | The counts object, in reply order.
countsOf :: QueryReq -> [Clause] -> Maybe Checked -> Tally -> [(String, Integer)]
countsOf req clauses chk tally =
  [ ("rules", toInteger (length [() | Rule {} <- clauses]))
  , ("queries", toInteger (length [() | Query {} <- clauses]))
  , ("asserts", toInteger (length [() | Assert {} <- clauses]))
  , ("strata", maybe 0 (toInteger . length . chkStrata) chk)
  , ("facts", toInteger (sum [length rows | (_, rows) <- facts req]))
  , ("derived", tDerived tally)
  , ("answers", tAnswers tally)
  , ("violations", tViolations tally)
  , ("proofNodes", tNodes tally)
  , ("proofTruncated", tSkipped tally)
  ]

-- | Over-cap: a complete degraded reply with empty tables — a
-- program the core refused to judge has no answers and no errors.
degraded :: String -> QueryReq -> B8.ByteString
degraded proto req = reply proto req (blank req [] Nothing zeroTally True)

-- | The query.result object.
reply :: String -> QueryReq -> Answered -> B8.ByteString
reply proto req a =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("query.result" :: String)
    , "id" .= reqId req
    , "goals" .= ansGoals a
    , "preds" .= ansPreds a
    , "answers" .= ansRows a
    , "proof" .= ansProof a
    , "errors" .= ansErrors a
    , "counts" .= object [Key.fromString key .= v | (key, v) <- ansCounts a]
    , "degraded" .= ansDegraded a
    ]
      <> ["reason" .= ("query_too_large" :: String) | ansDegraded a]
      <> ["schema" .= schemaRows | schemaOf req]

-- | The fact tables as the database the evaluation starts from.
factsDb :: QueryReq -> Db
factsDb req = IM.fromList [(code, S.fromList rows) | (code, rows) <- facts req]
