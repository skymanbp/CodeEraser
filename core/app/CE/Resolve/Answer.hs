{-# LANGUAGE OverloadedStrings #-}

-- | One site's answer (plan v2.33 wave W2a; on text since W2-text) —
-- the measuring side's `ladder::Outcome` for the ladders this family
-- holds: a file at a rung, a Go package directory at a rung, a file
-- reached through a Rust re-export surface at a rung (`ResolvedVia`,
-- plan v2.33 W2-text stage F), a Markdown section of a file at a rung
-- (`ResolvedSection`, its slug when the anchor was confirmed) and a
-- Markdown definition's inert answer (`ResolvedInert`, both stage G),
-- External at a rung, or a refusal with its reason — and the reply row
-- it travels as, `[rung, outcome, target, reason]`: the target is the
-- file's or the directory's repo-relative path, null where absent; the
-- reason -1 where absent.
module CE.Resolve.Answer (
  Answer (..),
  withRung,
  oneOf,
  firstOf,
  answerRow,
  holderOf,
) where

import CE.Resolve.Cost
import Data.Aeson (Value (..), toJSON)
import qualified Data.Set as Set

data Answer
  = AFile String !Int
  | APackage String !Int
  | AVia String !Int
  | ASection String (Maybe String) !Int
  | AInert String !Int
  | AExternal !Int
  | AUnresolved !Reason
  deriving (Eq, Show)

-- | `Outcome::with_rung`: the same answer at another rung; a refusal
-- passes untouched.
withRung :: Int -> Answer -> Answer
withRung r a = case a of
  AFile f _ -> AFile f r
  APackage d _ -> APackage d r
  AVia f _ -> AVia f r
  ASection f s _ -> ASection f s r
  AInert f _ -> AInert f r
  AExternal _ -> AExternal r
  AUnresolved why -> AUnresolved why

-- | `paths::one_of`: a rung's distinct in-scope candidates as its
-- answer — none leaves the next rung to ask, one resolves at the rung,
-- two or more is one name in two places.
oneOf :: Set.Set String -> Int -> Maybe Answer
oneOf hits rung = case Set.toList hits of
  [] -> Nothing
  [p] -> Just (AFile p rung)
  _ -> Just (AUnresolved AmbiguousRoot)

-- | The first rung that answers (an `Option::or_else` chain), else the
-- fallback.
firstOf :: [Maybe Answer] -> Answer -> Answer
firstOf rungs fallback = case [a | Just a <- rungs] of
  (a : _) -> a
  [] -> fallback

-- | A site whose answer is a confirmed section of ANOTHER file also
-- references that file (lane v235_licence, part 4C): one `also` row
-- `(target, rung)` at the answer's rung, the answer itself untouched. None
-- for a section of the linking file `from` itself (a page pointing into
-- itself is referenced by nothing), none for an unconfirmed fragment (its
-- answer is already the file), none for an inert answer.
holderOf :: String -> Answer -> [(String, Int)]
holderOf from a = case a of
  ASection target (Just _) r | target /= from -> [(target, r)]
  _ -> []

answerRow :: Answer -> Value
answerRow a = toJSON $ case a of
  AFile f r -> [toJSON r, toJSON outFile, toJSON f, none]
  APackage d r -> [toJSON r, toJSON outPackage, toJSON d, none]
  AVia f r -> [toJSON r, toJSON outVia, toJSON f, none]
  ASection f _ r -> [toJSON r, toJSON outSection, toJSON f, none]
  AInert f r -> [toJSON r, toJSON outInert, toJSON f, none]
  AExternal r -> [toJSON r, toJSON outExternal, Null, none]
  AUnresolved why -> [toJSON (0 :: Int), toJSON outUnresolved, Null, toJSON (reasonCode why)]
 where
  none = toJSON (-1 :: Int)
