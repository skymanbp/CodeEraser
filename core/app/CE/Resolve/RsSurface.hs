{-# LANGUAGE OverloadedStrings #-}

-- | What the Rust rungs read of a file's syntax tree, as the measuring
-- side's tree-sitter read it (plan v2.33 W2-text stage F; facts 4 and 5
-- of CE.Resolve.TsFacts), and the re-export surface's questions over it
-- (moved from cli/src/graph/ladder/rs_reexport.rs — `binds_to`, `exports`
-- and `owns` are the Rust functions of the same name). At one row
-- (`RsAt`): how many bodied `mod` items enclose it and their names,
-- outermost first; each `mod` item starting there with its `#[path]`
-- value; the `mod` items of the namespace enclosing it (the innermost
-- bodied mod's, else the file's), each bodied or not with its row; each
-- `use` item starting there with its argument's first line, trimmed, and
-- the whole argument, its whitespace folded. At the top level
-- (`Surface`): the item names with whether each is pub, and every use
-- and extern-crate binding flattened to (bound name, full path, row,
-- pub) — a glob under the `*` slot, a global path with an empty first
-- segment. A file that does not parse answers nothing.
module CE.Resolve.RsSurface (
  RsAt (..),
  Bind (..),
  rsAt,
  readsAsAt,
  readsAsSurface,
  bindsTo,
  exports,
  owns,
) where

import CE.Resolve.TsFacts (Fact (..), Facts, Need, payloadOf)
import Data.Aeson (FromJSON (..), Value, withObject, (.:))
import Data.Aeson.Types (parseMaybe)
import Data.Maybe (fromMaybe, isJust)

data RsAt = RsAt
  { raDepth :: Int
  , raMods :: [String]
  , raItems :: [(String, Maybe String)]
  , raNs :: [(String, Bool, Int)]
  , raUses :: [(String, String)]
  }

instance FromJSON RsAt where
  parseJSON = withObject "RsAt" $ \o -> RsAt <$> o .: "depth" <*> o .: "mods" <*> o .: "items" <*> o .: "ns" <*> o .: "uses"

data Surface = Surface
  { suDefs :: [(String, Bool)]
  , suUses :: [(String, [String], Int, Bool)]
  }

instance FromJSON Surface where
  parseJSON = withObject "Surface" $ \o -> Surface <$> o .: "defs" <*> o .: "uses"

-- | Whether a payload reads as a row's answers, as a surface (the
-- request contract's check).
readsAsAt, readsAsSurface :: Value -> Bool
readsAsAt v = isJust (parseMaybe parseJSON v :: Maybe RsAt)
readsAsSurface v = isJust (parseMaybe parseJSON v :: Maybe Surface)

-- | A file's answers at a row; a file that does not parse answers none.
rsAt :: Facts -> String -> Int -> Need RsAt
rsAt fx path row = fromMaybe (RsAt 0 [] [] [] []) . (>>= parseMaybe parseJSON) <$> payloadOf fx (FRsAt path row)

surface :: Facts -> String -> Need Surface
surface fx path = fromMaybe (Surface [] []) . (>>= parseMaybe parseJSON) <$> payloadOf fx (FRsSurface path)

-- | `GLOB`: the name slot of a glob entry.
glob :: String
glob = "*"

-- | What a surface says about a name: one entry binds it to a full path
-- (with the declaration's row), or no entry names it and these globs
-- (each the path before its `*`, with its row) may carry it.
data Bind = Named [String] Int | Globbed [([String], Int)]

-- | `pub_entries`: the pub bindings.
pubEntries :: Surface -> [(String, [String], Int)]
pubEntries s = [(n, p, r) | (n, p, r, True) <- suUses s]

-- | `binds_to`: none when the file defines the name at top level (the
-- definition wins), when two entries bind it, or when no glob is left.
bindsTo :: Facts -> String -> String -> Need (Maybe Bind)
bindsTo fx f name = verdict <$> surface fx f
 where
  verdict s
    | any ((== name) . fst) (suDefs s) = Nothing
    | otherwise = case [(p, r) | (n, p, r) <- pubEntries s, n == name] of
        [(p, r)] -> Just (Named p r)
        (_ : _ : _) -> Nothing
        [] -> case [(p, r) | (n, p, r) <- pubEntries s, n == glob] of
          [] -> Nothing
          globs -> Just (Globbed globs)

-- | `exports`: a pub item the file defines, or a pub use binding the
-- name.
exports :: Facts -> String -> String -> Need Bool
exports fx f name = verdict <$> surface fx f
 where
  verdict s = any (== (name, True)) (suDefs s) || any (\(n, _, _) -> n == name) (pubEntries s)

-- | `owns`: the name defined at top level, or bound by any top-level use
-- (a private one too).
owns :: Facts -> String -> String -> Need Bool
owns fx f name = verdict <$> surface fx f
 where
  verdict s = any ((== name) . fst) (suDefs s) || any (\(n, _, _, _) -> n == name) (suUses s)
