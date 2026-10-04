-- | One site's answer (plan v2.33 wave W2a) — the measuring side's
-- `ladder::Outcome` for the four ladders this family holds: a file at a
-- rung, a Go package directory at a rung, External at a rung, or a
-- refusal with its reason — and the reply row it travels as,
-- `[rung, outcome, target, reason]` with -1 where a column is absent.
module CE.Resolve.Answer (
  Answer (..),
  withRung,
  oneOf,
  firstOf,
  answerRow,
) where

import CE.Resolve.Cost

data Answer
  = AFile !Int !Int
  | APackage !Int !Int
  | AExternal !Int
  | AUnresolved !Reason
  deriving (Eq, Show)

-- | The same answer at another rung; a refusal passes untouched.
withRung :: Int -> Answer -> Answer
withRung r a = case a of
  AFile f _ -> AFile f r
  APackage d _ -> APackage d r
  AExternal _ -> AExternal r
  AUnresolved why -> AUnresolved why

-- | A rung's distinct in-scope files as its answer: none leaves the
-- next rung to ask, one resolves at the rung, two or more is one name
-- in two places.
oneOf :: [Int] -> Int -> Maybe Answer
oneOf files rung = case files of
  [] -> Nothing
  [f] -> Just (AFile f rung)
  _ -> Just (AUnresolved AmbiguousRoot)

-- | The first rung that answers, else the fallback.
firstOf :: [Maybe Answer] -> Answer -> Answer
firstOf rungs fallback = case [a | Just a <- rungs] of
  (a : _) -> a
  [] -> fallback

answerRow :: Answer -> [Integer]
answerRow a = case a of
  AFile f r -> [toInteger r, outFile, toInteger f, -1]
  APackage d r -> [toInteger r, outPackage, toInteger d, -1]
  AExternal r -> [toInteger r, outExternal, -1, -1]
  AUnresolved why -> [0, outUnresolved, -1, reasonCode why]
