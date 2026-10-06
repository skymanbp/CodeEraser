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
  langRs,
  langTs,
  langTsx,
  langGo,
  langC,
  langCpp,
  langLua,
  langR,
  langJava,
  langHs,
  langMd,
  resolvedLangs,
  outFile,
  outPackage,
  outExternal,
  outUnresolved,
  outVia,
  outSection,
  outInert,
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

-- | The language codes of the ladders this family holds (the
-- `languages.rows` codes): Python, TypeScript, TSX, Rust, Go, Markdown,
-- Haskell, C, C++, Lua, Java, R. Every other
-- language's ladder still runs on the measuring side during the track.
langPy, langTs, langTsx, langRs, langGo, langMd, langHs, langC, langCpp, langLua, langJava, langR :: Integer
(langPy, langTs, langTsx, langRs, langGo, langMd, langHs, langC, langCpp, langLua, langJava, langR) = (0, 1, 2, 3, 4, 5, 6, 15, 16, 17, 18, 20)

resolvedLangs :: [Integer]
resolvedLangs = [langPy, langTs, langTsx, langRs, langGo, langMd, langHs, langC, langCpp, langLua, langJava, langR]

-- | The outcome column of a reply row: a file target, a package
-- directory target (Go, R, Java), External, Unresolved, a file target
-- reached through a Rust re-export surface (the measuring side's
-- `ResolvedVia`, since plan v2.33 W2-text stage F), a Markdown section
-- (`ResolvedSection`; its slug rides in the reply's `sections` table)
-- and a Markdown definition's inert answer (`ResolvedInert`, both since
-- stage G).
outFile, outPackage, outExternal, outUnresolved, outVia, outSection, outInert :: Integer
outFile = 0
outPackage = 1
outExternal = 2
outUnresolved = 3
outVia = 4
outSection = 5
outInert = 6

-- | The refusal vocabulary, in the measuring side's declaration order
-- (its codes are this order's indices). The ladders here answer nine
-- of them (Java added ambiguous_paths and own_unit, the TS rungs
-- ambiguous_exports and config_depth); the rest belong to
-- ladders still on the measuring side.
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
