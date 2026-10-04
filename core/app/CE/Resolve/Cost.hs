-- | The resolve family's constants (plan v2.33 wave W2a; design booklet
-- docs/reference/algorithm-track.md §6 row W2): the one cap every table
-- of a request counts against, the language codes the family answers,
-- the outcome and reason codes a reply row carries, and the token
-- separators a specifier row may hold. The codes are the measuring
-- side's `ladder::Outcome` / `ladder::Reason` in declaration order —
-- the reply is exactly what an outcome carried when the search lived
-- in cli/src/graph/ladder/, so everything downstream of it (the edge
-- store, deadcode, `ce graph --sites`, the precision documents) reads
-- the same answers.
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
  sepDot,
  sepSlash,
  formSystem,
  formRooted,
  formDotted,
  formPath,
  forcedRelative,
  forcedPlaced,
  forcedOutside,
) where

-- | Every row of every table of one request, together: segments are
-- not rows (a count), the vocabulary is the core's own length. Over
-- the cap answers a complete degraded reply, never a partial one —
-- a tree that large is refused by name on the measuring side.
resolveCap :: Integer
resolveCap = 4194304

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

-- | Token separators inside a specifier row: Python's dot between two
-- dotted segments, Lua's dot and slash between two name pieces. Every
-- other token is a segment id (non-negative).
sepDot, sepSlash :: Integer
sepDot = -1
sepSlash = -2

-- | The `form` column's bits, per language: C's `<…>` form, a Lua
-- `dofile` path that names no file of the tree (rooted at `/` or `~`,
-- or carrying a drive or a colon), a Go import or replacement whose
-- first segment holds a dot (a host name), a Go replacement that is a
-- filesystem path (`./`, `../`). Python's form is its leading-dot count.
formSystem, formRooted, formDotted, formPath :: Integer
formSystem = 1
formRooted = 1
formDotted = 1
formPath = 2

-- | A forced include's first column: a spelling to search, an absolute
-- path the measuring side placed into the tree, or one placed outside it.
forcedRelative, forcedPlaced, forcedOutside :: Integer
forcedRelative = 0
forcedPlaced = 1
forcedOutside = 2
