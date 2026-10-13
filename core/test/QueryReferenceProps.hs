-- | The query family's generated battery (design booklet §4.7): every
-- program of ReferenceQueryGen over its fact set through the shipped
-- parser, checker and semi-naive evaluator and through ReferenceQuery
-- — a program the reference cannot stratify must be refused at its
-- head as unstratifiable, any other must be checked, and every program
-- predicate's tuples and every goal's answers must agree — and the
-- shapes the two hundred programs must reach between them.
module QueryReferenceProps (equivalence) where

import CE.Query.Check (Checked (..), check)
import CE.Query.Cost (errUnstratified)
import CE.Query.Eval (evalProgram, goalAnswers)
import CE.Query.Parse (parseProgram)
import CE.Query.Syntax (isIdb)
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S
import ReferenceQuery (events, goalAnswersRef, naive, strataCount, toks)
import ReferenceQueryGen (QCase (..), cases)
import WireHarness (generated, runChecks)

equivalence :: IO Bool
equivalence = runChecks (generated names agrees reached shapes cases)
 where
  names = ("two hundred generated programs agree with the naive reference: refusals, program predicates and goal answers", "the generated programs reach every shape §4.1 states")

agrees :: QCase -> Maybe String
agrees c = case parseProgram (toks (qText c)) of
  Left e -> Just ("parse " <> show e <> ": " <> qText c)
  Right clauses -> case (naive clauses (qFacts c), check 0 clauses) of
    (Nothing, Left errs) | not (null errs), all ((== errUnstratified) . snd) errs -> Nothing
    (Just want, Right chk) -> case evalProgram chk (qFacts c) of
      Right (got, _, _)
        | idb got == idb want && map (map fst . goalAnswers got) (chkGoals chk) == goalAnswersRef clauses want -> Nothing
      _ -> Just ("disagree: " <> qText c)
    (r, k) -> Just ("refusal " <> show (fmap (const ()) r) <> " vs " <> either show (const "checked") k <> ": " <> qText c)
 where
  idb = IM.filterWithKey (\p _ -> isIdb p)

reached :: QCase -> S.Set String
reached c = S.fromList $ case parseProgram (toks (qText c)) of
  Left _ -> []
  Right clauses -> qTags c <> case naive clauses (qFacts c) of
    Nothing -> ["an unstratifiable program refused"]
    Just db ->
      events clauses db
        <> ["three strata or more" | maybe False (>= 3) (strataCount clauses)]
        <> [if null (last answers) then "an assertion that holds" else "an assertion with violations" | qAssert c, let answers = goalAnswersRef clauses db, not (null answers)]

shapes :: [String]
shapes =
  [op <> over | op <- ["count", "min", "max", "sum"], over <- [" over rows", " over nothing"]]
    <> ["operator " <> op | op <- ["+", "-", "*", "/", "%"]]
    <> ["a division by zero fails its literal", "a bound = compares"]
    <> ["comparison " <> op | op <- ["!=", "<", "<=", ">", ">="]]
    <> ["recursion", "negation", "three strata or more", "an unstratifiable program refused", "an assertion that holds", "an assertion with violations"]
