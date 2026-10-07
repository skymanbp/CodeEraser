-- | The Markdown rungs (plan v2.33 W2-text stage G; once
-- cli/src/graph/ladder/md.rs; design §4 row 5): doc targets resolve at
-- file and section granularity. R1 joins a relative target, decoded
-- after the split at `#` (an encoded `%23` is a literal `#` of the
-- name), against the linking file's directory — a walked file, or a
-- walked asset the index never parsed; a directory holding walked files
-- resolves as a package (a tree reference collapsed to one file would
-- be a guess). R2 checks a cross-file fragment, decoded, against the
-- target's anchor set as the measuring side read it (the rendered
-- heading slugs and raw-HTML anchor ids, `md.slugs`): exactly one match
-- names the section, anything else degrades to the file (no slug). R3 a
-- reference link substitutes its file's definition (CommonMark: labels
-- fold — whitespace collapsed, Rust's lowercase — and the first
-- definition wins) and reruns the chain as rung 3; a definition no
-- reference link uses still resolves but travels inert (the liveness
-- exclusion, user decision D3). R4 a bare fragment is a section claim on
-- the linking document taken as written. R5 a URI scheme or a
-- protocol-relative form is External; a site-root `/x` is
-- domain-relative, never in the tree. What the measuring side read of
-- the text — the heading slugs, the definitions outside code and
-- comments, the labels its reference links use — arrives as facts; the
-- masking, the heading walk and the slugging stay there (their other
-- readers, the tombstone leg on the hook path, docdup's segments,
-- fourclass's section units and the walk's key, have no core
-- replacement yet).
module CE.Resolve.Md (MdEnv (..), anchor, fold, fragment, refTable, resolveMd) where

import CE.Resolve.Answer
import CE.Resolve.Cost
import CE.Resolve.Lower (rustLower)
import CE.Resolve.Str (joinRel, parentDir, splitWhitespace)
import CE.Resolve.Tables (kindImage, kindLink, kindRefDef, kindRefLink, kindUrl)
import CE.Resolve.Url (isScheme, percentDecode)
import CE.Resolve.World (World (..), member, ofLangs)
import Data.List (intercalate, isPrefixOf)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | The walked tree, its walked assets, each walked Markdown file's
-- anchor set and each reference site's file's reference table.
data MdEnv = MdEnv
  { mdWorld :: World
  , mdAssets :: Set.Set String
  , mdSlugs :: M.Map String [String]
  , mdRefs :: M.Map String RefTable
  }

-- | A file's reference surface: the first definition of each folded
-- label, and the folded labels its reference links use.
type RefTable = (M.Map String String, Set.Set String)

-- | The table of a file's definitions `(label, target)` in text order
-- and the labels its reference links write: a later definition of a
-- label already defined is inert.
refTable :: [(String, String)] -> [String] -> RefTable
refTable defs used = (M.fromListWith (\_ first -> first) [(fold l, t) | (l, t) <- defs], Set.fromList (map fold used))

-- | CommonMark's label normalization as the measuring side spelled it:
-- `split_whitespace`, joined by one space, `to_lowercase`.
fold :: String -> String
fold = rustLower . intercalate " " . splitWhitespace

-- | One site by its kind: a link and an image share the chain (an
-- image's asset edge is wiring, not resolution), a URL is External.
resolveMd :: MdEnv -> Integer -> String -> String -> Answer
resolveMd env kind from spec
  | kind == kindLink || kind == kindImage = link env from spec
  | kind == kindRefLink = maybe (AUnresolved OutOfScope) (withRung 3 . link env from) (M.lookup (fold spec) defs)
  | kind == kindRefDef = refDef env from spec
  | kind == kindUrl = AExternal 5
  | otherwise = AUnresolved Unsupported
 where
  (defs, _) = refsOf env from

refsOf :: MdEnv -> String -> RefTable
refsOf env from = M.findWithDefault (M.empty, Set.empty) from (mdRefs env)

-- | R1 / R2 / R4 / R5 for a direct target.
link :: MdEnv -> String -> String -> Answer
link env from spec
  | isScheme spec = AExternal 5
  | "/" `isPrefixOf` spec = AUnresolved OutOfScope
  | null path = fragment from frag
  | otherwise = maybe (AUnresolved OutOfScope) (located env frag) (joinRel (parentDir from) (percentDecode path))
 where
  (path, frag) = case break (== '#') spec of
    (p, '#' : f) -> (p, Just f)
    _ -> (spec, Nothing)

-- | R1 / R2 for a joined target: a walked Markdown file with a fragment
-- checks its anchor, a walked file or asset is the file, else a
-- directory.
located :: MdEnv -> Maybe String -> String -> Answer
located env frag target
  | member w target = case frag of
      Just f | ofLangs ["markdown"] target && not (null f) -> anchor env target f
      _ -> AFile target 1
  | Set.member target (mdAssets env) = AFile target 1
  | otherwise = directory w target
 where
  w = mdWorld env

-- | R4: a section of the linking document, as written.
fragment :: String -> Maybe String -> Answer
fragment from frag = case frag of
  Just f | not (null f) -> ASection from (Just f) 4
  _ -> AFile from 4

-- | R1's directory arm: a package while walked files live under it.
directory :: World -> String -> Answer
directory w target = case Set.lookupGE prefix (wFiles w) of
  Just f | prefix `isPrefixOf` f -> APackage target 1
  _ -> AUnresolved OutOfScope
 where
  prefix = if null target then "" else target <> "/"

-- | R2: the decoded fragment against the target's anchor set.
anchor :: MdEnv -> String -> String -> Answer
anchor env target frag = ASection target (if hits == 1 then Just decoded else Nothing) 2
 where
  decoded = percentDecode frag
  hits = length (filter (== decoded) (M.findWithDefault [] target (mdSlugs env)))

-- | A definition site: used, its target resolves like a link; unused,
-- the answer travels inert (a refusal stays a refusal).
refDef :: MdEnv -> String -> String -> Answer
refDef env from spec
  | or [t == spec && Set.member label used | (label, t) <- M.toList defs] = resolved
  | otherwise = case resolved of
      AFile p r -> AInert p r
      ASection p _ r -> AInert p r
      APackage d r -> AInert d r
      other -> other
 where
  (defs, used) = refsOf env from
  resolved = withRung 3 (link env from spec)
