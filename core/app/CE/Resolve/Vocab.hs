{-# LANGUAGE OverloadedStrings #-}

-- | The words the resolve family's search spells (plan v2.33 wave W2a):
-- the product's own path words (`__init__.py`, `src`, `init.lua`,
-- `include`, a framework's `Headers` …), the suffixes it appends
-- (`.py`, `.lua`, `.framework`) or tests (`.go`, `_test.go`, a
-- translation unit's `.c` / `.cc` / `.cpp` / `.cxx`), and every piece
-- of the three standard-library tables the External rungs read
-- (`ladder.py.stdlib`, `ladder.lua.stdlib`, `ladder.go.std` of the
-- definition package). No repository string ever reaches the core, so
-- the measuring side interns exactly this list — the package carries
-- it as `resolve.words` and `resolve.affixes` — and a request sends one
-- segment id per entry, in this order. A word the repository never
-- spells still gets an id: it can match no file, which is the answer.
module CE.Resolve.Vocab (
  words,
  affixes,
  vocabLength,
  Word' (..),
  wordAt,
  codeOf,
  Affix (..),
  affixAt,
  pyStdlib,
  luaStdlib,
  goStd,
  kindRequire,
  kindLoad,
  table,
) where

import CE.Lang (pack, siteKinds)
import Data.Aeson (Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseEither, parseJSON)
import Data.List (elemIndex)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import Prelude hiding (words)

-- | The fixed words, by position in `words`.
data Word'
  = WEmpty
  | WDot
  | WDotDot
  | WInitPy
  | WSrc
  | WFuture
  | WLua
  | WInitLua
  | WInclude
  | WHeaders
  | WPrivateHeaders
  deriving (Eq, Show, Enum, Bounded)

fixed :: [String]
fixed = ["", ".", "..", "__init__.py", "src", "__future__", "lua", "init.lua", "include", "Headers", "PrivateHeaders"]

-- | The fixed words, then every standard-library piece once (first
-- occurrence kept, so a fixed word keeps its position).
words :: [String]
words = fixed <> filter (`Set.notMember` Set.fromList fixed) (dedup (concat (pyStdlib <> luaStdlib <> goStd)))
 where
  dedup = go Set.empty
  go _ [] = []
  go seen (x : xs)
    | Set.member x seen = go seen xs
    | otherwise = x : go (Set.insert x seen) xs

-- | The suffixes, by position in `affixes`.
data Affix = APy | ALua | AFramework | AGo | AGoTest | AC | ACc | ACpp | ACxx
  deriving (Eq, Enum, Bounded)

affixes :: [String]
affixes = [".py", ".lua", ".framework", ".go", "_test.go", ".c", ".cc", ".cpp", ".cxx"]

-- | The request's `vocab` length: the words, then the affixes.
vocabLength :: Int
vocabLength = length words + length affixes

-- | A fixed word's position in the request's `vocab`.
wordAt :: Word' -> Int
wordAt = fromEnum

-- | Any word's position in the request's `vocab` (the standard-library
-- pieces are words too).
codeOf :: String -> Int
codeOf s = M.findWithDefault (error ("not a resolve word: " <> s)) s codes

codes :: M.Map String Int
codes = M.fromList (reverse (zip words [0 ..]))

-- | An affix's position in the request's `vocab`.
affixAt :: Affix -> Int
affixAt a = length words + fromEnum a

-- | The External tables, each entry as its pieces: Python's top-level
-- module names (one piece), Lua's names split at their dots, Go's
-- import paths split at their slashes.
pyStdlib, luaStdlib, goStd :: [[String]]
pyStdlib = map pure (ladderNames "py" "stdlib")
luaStdlib = map (splitOn '.') (ladderNames "lua" "stdlib")
goStd = map (splitOn '/') (ladderNames "go" "std")

splitOn :: Char -> String -> [String]
splitOn c s = case break (== c) s of
  (a, _ : rest) -> a : splitOn c rest
  (a, []) -> [a]

-- | `ladder.<lang>.<key>` of the definition package.
ladderNames :: String -> String -> [String]
ladderNames lang key = case pack of
  Object o
    | Just (Object l) <- KM.lookup "ladder" o
    , Just (Object t) <- KM.lookup (K.fromString lang) l
    , Just v <- KM.lookup (K.fromString key) t ->
        either refuse id (parseEither parseJSON v)
  _ -> refuse ("no ladder." <> lang <> "." <> key)
 where
  refuse e = error ("resolve vocabulary does not read: " <> e)

-- | The storage codes of the two Lua site kinds the Lua ladder branches
-- on (the `store.site_kinds` table).
kindRequire, kindLoad :: Integer
kindRequire = kindCode "require"
kindLoad = kindCode "load"

kindCode :: String -> Integer
kindCode k = maybe (error ("no site kind " <> k)) toInteger (elemIndex k siteKinds)

-- | The two lists as the package carries them (`resolve`).
table :: Value
table = Object (KM.fromList [("words", toJSON words), ("affixes", toJSON affixes)])
