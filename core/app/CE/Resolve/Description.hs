-- | An R package's DESCRIPTION (Writing R Extensions §1.1.1; moved from
-- cli/src/graph/ladder/r/description.rs and the package-code expansion
-- of cli/src/graph/deadcode/targets.rs in plan v2.33 W2-text stage B —
-- each function below is the Rust function of the same name), read for
-- the two fields the graph needs: `Package`, the name `library()` and
-- `pkg::` reach the package by (the R ladder's R2), and `Collate`, the
-- order R loads the package's `R/` files in — so the files the package
-- declares (the declared-target role). The format is Debian control:
-- `Field: value`, the value continued on each following line that
-- starts with a space or a tab. The measuring side reads the file lossy
-- (a DESCRIPTION may declare `Encoding: latin1`; the two fields read here
-- are ASCII either way) and sends the text.
module CE.Resolve.Description (
  Description (..),
  readDescription,
  packageCode,
) where

import CE.Resolve.Chars (isRustWhite)
import CE.Resolve.Str (joinDir, joinRel, parentDir, rustLines, rustTrim)
import CE.Resolve.World (World (..), member, ofLangs)
import Data.List (intercalate, stripPrefix)
import qualified Data.Set as Set

-- | One DESCRIPTION: the directory holding it — the package root, `""`
-- at the tree root — the package's name, and its `Collate` list (file
-- names under `R/`; empty when the field is absent, and R then loads
-- every file of `R/`).
data Description = Description
  { dDir :: String
  , dPackage :: String
  , dCollate :: [String]
  }
  deriving (Eq, Show)

-- | `read`: one DESCRIPTION's text, the file held in `dir`. A package
-- name is one word; anything else names no package.
readDescription :: String -> String -> Maybe Description
readDescription dir text = do
  package <- rustTrim <$> field "Package" text
  if null package || any isRustWhite package
    then Nothing
    else Just (Description dir package (maybe [] names (field "Collate" text)))

-- | `field`: a field's value with its continuation lines, joined by
-- newlines; Nothing when the text has no such field. `Packaged:` is not
-- `Package:`.
field :: String -> String -> Maybe String
field name = go . rustLines
 where
  go [] = Nothing
  go (l : ls) = case stripPrefix name l >>= stripPrefix ":" of
    Nothing -> go ls
    Just value -> Just (intercalate "\n" (value : takeWhile continued ls))
  continued l = case l of
    (c : _) -> c == ' ' || c == '\t'
    [] -> False

-- | `names`: the file names a `Collate` value lists — separated by white
-- space, each optionally quoted with `'` or `"` (a quoted name may hold
-- a space; an unclosed quote runs to the end).
names :: String -> [String]
names value = case dropWhile isRustWhite value of
  [] -> []
  (q : rest) | q == '\'' || q == '"' -> let (n, after) = break (== q) rest in keep n (names (drop 1 after))
  t -> let (n, after) = break isRustWhite t in keep n (names after)
 where
  keep n more = if null n then more else n : more

-- | `package_code`: an R package's code — its `Collate` files under `R/`
-- when it lists them, else every walked R file directly in `R/` — what R
-- loads with the package, which nothing `source`s; in path order.
packageCode :: World -> Description -> [String]
packageCode w d
  | null (dCollate d) = [f | f <- Set.toList (wFiles w), ofLangs ["r"] f, parentDir f == code]
  | otherwise = Set.toList (Set.fromList [p | n <- dCollate d, Just p <- [joinRel code n], member w p])
 where
  code = joinDir (dDir d) "R"
