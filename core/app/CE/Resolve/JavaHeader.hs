-- | A Java compilation unit's header as the measuring side's walk read
-- it (plan v2.33 W2-text stage C): cli/src/graph/ladder/java_header.rs
-- lexes each walked Java file's head — the package it declares and the
-- imports it writes, each with the line of its `import` keyword — and
-- java_types.rs scans its type declarations (simple name, supertypes as
-- written, member types, first and last line). That reading stays on the
-- measuring side, where the walk holds the text; the rows it sends are
-- what the Java rungs read of it:
--   header  [path, package, [import], [type]]
--   import  [name, star, static, line]
--   type    [name, [super], [type], first, last]
module CE.Resolve.JavaHeader (
  JHeader (..),
  JImport (..),
  JType (..),
  headerRow,
) where

import Data.Aeson (FromJSON (..), Value)
import Data.Aeson.Types (Parser)

data JHeader = JHeader
  { hPackage :: String
  , hImports :: [JImport]
  , hTypes :: [JType]
  }

data JImport = JImport
  { iName :: String
  , iStar :: Bool
  , iStatic :: Bool
  , iLine :: Int
  }

data JType = JType
  { tName :: String
  , tSupers :: [String]
  , tMembers :: [JType]
  , tFirst :: Int
  , tLast :: Int
  }

-- | One header row: its path and the header.
headerRow :: Value -> Parser (String, JHeader)
headerRow v = do
  (path, package, imports, types) <- parseJSON v
  pure (path, JHeader package imports types)

instance FromJSON JImport where
  parseJSON v = (\(n, s, st, l) -> JImport n s st l) <$> parseJSON v

instance FromJSON JType where
  parseJSON v = (\(n, ss, ms, a, b) -> JType n ss ms a b) <$> parseJSON v
