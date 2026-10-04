-- | The resolve family's constants (plan v2.33 wave W2a; on text since
-- W2-text; design booklet docs/reference/algorithm-track.md §6): the
-- one cap a request's text counts against, the language codes the
-- family answers, the outcome and reason codes a reply row carries. The
-- codes are the measuring side's `ladder::Outcome` / `ladder::Reason`
-- in declaration order — the reply is exactly what an outcome carried
-- when the search lived in cli/src/graph/ladder/, so everything
-- downstream of it (the edge store, deadcode, `ce graph --sites`, the
-- precision documents) reads the same answers.
module CE.Resolve.Cost (
  resolveCap,
  langPy,
  langGo,
  langC,
  langCpp,
  langLua,
  resolvedLangs,
  outFile,
  outPackage,
  outExternal,
  outUnresolved,
  Reason (..),
  reasonCode,
) where

-- | The characters of every string of one request plus one per row,
-- together (CE.Resolve.Contract.requestSize). Over the cap answers a
-- complete degraded reply, never a partial one — a tree that large is
-- refused by name on the measuring side. 256 Mi: the integer-row
-- family's cap was 4 Mi rows, and a path, a specifier or a
-- configuration line is a few dozen characters.
resolveCap :: Integer
resolveCap = 268435456

-- | The language codes of the four ladders this family holds (the
-- `languages.rows` codes): Python, Go, C, C++, Lua. Every other
-- language's ladder still runs on the measuring side during the track.
langPy, langGo, langC, langCpp, langLua :: Integer
langPy = 0
langGo = 4
langC = 15
langCpp = 16
langLua = 17

resolvedLangs :: [Integer]
resolvedLangs = [langPy, langGo, langC, langCpp, langLua]

-- | The outcome column of a reply row: a file target, a package
-- directory target (Go), External, Unresolved.
outFile, outPackage, outExternal, outUnresolved :: Integer
outFile = 0
outPackage = 1
outExternal = 2
outUnresolved = 3

-- | The refusal vocabulary, in the measuring side's declaration order
-- (its codes are this order's indices). The four ladders here answer
-- five of them; the rest belong to ladders still on the measuring side.
data Reason
  = Dynamic
  | AmbiguousPaths
  | AmbiguousRoot
  | AmbiguousWorkspace
  | AmbiguousExports
  | Macro
  | ConfigDepth
  | OutOfScope
  | Unsupported
  | Empty
  | OwnUnit
  deriving (Eq, Show, Enum, Bounded)

reasonCode :: Reason -> Integer
reasonCode = toInteger . fromEnum
