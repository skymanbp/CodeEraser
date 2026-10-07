-- | The HTML rungs (plan v2.33 W2-text stage H; once
-- cli/src/graph/ladder/html.rs; booklet §8, HTML row). A page names what
-- the site serves, so a target is any walked path — a page, a document,
-- a code file, or a walked asset the index never holds — and a directory
-- serves its index page. R1 joins a relative reference onto the
-- document's directory (the directory its `<base href>` names, when it
-- declares one). A root-relative reference (`/x`) needs the deployment
-- root, which no join knows: R2 takes it from the declared
-- `[graph.search_roots] html` directories (two answering is
-- ambiguous_root), or from the document's own served URL
-- (CE.Resolve.HtmlHead); R3, with neither, asks each ancestor directory
-- of the page and answers when exactly one holds the path (two is
-- ambiguous_root, none out_of_scope). An absolute URL on the document's
-- own host is that page's root-relative path; any other host or scheme
-- is External (R5, the Markdown reading). R4 is Markdown's too: a bare
-- fragment claims a section of the page as written, `#` alone the page.
-- A cross-page fragment is checked against the target page's `id` set
-- (a Markdown target's anchor set), zero or many matches degrading to
-- the file. The query is dropped, character and percent escapes decode
-- before any lookup, and an empty reference is the document's own URL:
-- a page or form reference lands the page, an empty `src`, `srcset` or
-- asset `<link>` fetches nothing and is refused as empty.
module CE.Resolve.Html (HtmlEnv (..), pageEnv, resolveHtml) where

import CE.Resolve.Answer
import CE.Resolve.Cost
import CE.Resolve.HtmlHead (Doc (..), Page (..), decodeRefs, page, splitOrigin)
import CE.Resolve.Md (MdEnv (..), anchor, fragment, refTable)
import CE.Resolve.Request (ResolveReq (..))
import CE.Resolve.Str (joinDir, joinRel, parentDir)
import CE.Resolve.Tables (kindAction, kindHref)
import CE.Resolve.Url (isScheme, percentDecode)
import CE.Resolve.World (World (..), member, ofLangs)
import Data.List (find, isPrefixOf, stripPrefix)
import qualified Data.Map.Strict as M
import Data.Maybe (mapMaybe, maybeToList)
import qualified Data.Set as Set

-- | The Markdown environment (the walked tree, its assets, the Markdown
-- anchor sets), each HTML document's facts by path, and the declared
-- `html` search roots (absent apart from declared empty).
data HtmlEnv = HtmlEnv
  { hMd :: MdEnv
  , hDocs :: M.Map String Doc
  , hRoots :: Maybe [String]
  }

-- | A request's document environment: the walked tree, its assets, each
-- Markdown document's anchor set and reference table, each page's facts
-- by path, and the declared `html` roots.
pageEnv :: World -> ResolveReq -> HtmlEnv
pageEnv w rq = HtmlEnv md (M.fromList [(p, Doc l b c o alts ids) | (p, l, b, c, o, alts, ids) <- rqHtmlDocs rq]) (M.lookup "html" (rqSearch rq))
 where
  md = MdEnv w (Set.fromList (rqAssets rq)) (M.fromList (rqMdSlugs rq)) (M.fromList [(f, refTable ds us) | (f, ds, us) <- rqMdRefs rq])

-- | A rung's answer: the found path and its rung, or the answer that
-- stands instead (a refusal, or External for a reference a `<base>` on
-- another host sends off the site).
type Found = Either Answer (String, Int)

-- | One site by its kind and specifier.
resolveHtml :: HtmlEnv -> Integer -> String -> String -> Answer
resolveHtml env kind from spec
  | null spec = if kind == kindHref || kind == kindAction then fragment from Nothing else AUnresolved Empty
  | Just (host, path) <- splitOrigin spec' = if pHost doc /= Just host then AExternal 5 else located env from (if null path then "/" else path) doc
  | isScheme spec' = AExternal 5
  | otherwise = located env from spec' doc
 where
  spec' = decodeRefs spec
  doc = pageOf env from

-- | A document's page facts; one the request did not carry says nothing.
pageOf :: HtmlEnv -> String -> Page
pageOf env from = maybe (Page Nothing Nothing Nothing) (page from) (M.lookup from (hDocs env))

-- | A reference without an origin: the fragment split off at the first
-- `#`, the query dropped at the first `?`, the path found (R1–R3) and
-- the fragment settled.
located :: HtmlEnv -> String -> String -> Page -> Answer
located env from reference doc
  | null path = fragment from frag
  | otherwise = either id (\(target, rung) -> sectioned env target frag rung) found
 where
  (written, frag) = case break (== '#') reference of
    (p, '#' : f) -> (p, Just f)
    _ -> (reference, Nothing)
  path = takeWhile (/= '?') written
  found = case stripPrefix "/" path of
    Just rel -> rooted env from rel doc (candidate env)
    Nothing -> relative env from path doc

-- | R1: the reference, percent-decoded, joined onto the document's
-- directory — its `<base href>`'s first.
relative :: HtmlEnv -> String -> String -> Page -> Found
relative env from path doc = do
  dir <- maybe (Right (parentDir from)) (\b -> baseDir env from b doc) (pBase doc)
  target <- maybe (Left outOfScope) Right (joinRel dir (percentDecode path))
  maybe (Left outOfScope) (\t -> Right (t, 1)) (candidate env target)

-- | The directory a `<base href>` names: on another host External, a
-- same-host or root-relative one under the deployment root (its path's
-- directory part, R2 / R3 with a directory as the hit), a relative one
-- beside the document; out of scope when no walked file lives under it.
baseDir :: HtmlEnv -> String -> String -> Page -> Either Answer String
baseDir env from base doc
  | Just (host, path) <- splitOrigin base = if pHost doc /= Just host then Left (AExternal 5) else underRoot (dropWhile (== '/') path)
  | isScheme base = Left (AExternal 5)
  | Just rel <- stripPrefix "/" base = underRoot rel
  | otherwise = maybe (Left outOfScope) Right (joinRel (parentDir from) (parentDir base) >>= exists)
 where
  exists d = if directory env d then Just d else Nothing
  underRoot rel = fst <$> rooted env from (parentDir rel) doc exists

-- | R2 / R3 for a root-relative path, percent-decoded: under the declared
-- `html` roots, else under the root the page's own URL derives (both
-- rung 2), else under each ancestor directory of the page (rung 3);
-- `hit` is what holds it — a walked file or index page for a reference,
-- a directory for a base.
rooted :: HtmlEnv -> String -> String -> Page -> (String -> Maybe String) -> Found
rooted env from rel doc hit = case (hRoots env, pRoot doc) of
  (Just dirs, _) -> unique (mapMaybe under dirs) 2
  (Nothing, Just root) -> unique (maybeToList (under root)) 2
  _ -> unique (mapMaybe under ancestors) 3
 where
  decoded = percentDecode rel
  under d = joinRel d decoded >>= hit
  dir = parentDir from
  ancestors = [take i dir | (i, '/') <- zip [0 ..] dir] <> [dir, ""]

-- | A rung's distinct hits as its answer: one answers, two is one path
-- in two roots, none is out of scope.
unique :: [String] -> Int -> Found
unique hits rung = case Set.toList (Set.fromList hits) of
  [one] -> Right (one, rung)
  [] -> Left outOfScope
  _ -> Left (AUnresolved AmbiguousRoot)

-- | The walked path a tree path serves: itself (a walked file or asset),
-- or a directory's walked index page.
candidate :: HtmlEnv -> String -> Maybe String
candidate env path
  | member w path || Set.member path (mdAssets (hMd env)) = Just path
  | otherwise = find (member w) [joinDir path "index.html", joinDir path "index.htm"]
 where
  w = mdWorld (hMd env)

-- | Whether walked files or assets live under a directory (the tree
-- root always).
directory :: HtmlEnv -> String -> Bool
directory env dir = null dir || any holds [wFiles (mdWorld md), mdAssets md]
 where
  md = hMd env
  prefix = dir <> "/"
  holds set = maybe False (prefix `isPrefixOf`) (Set.lookupGE prefix set)

-- | The fragment on a found target: a Markdown document's anchor set or
-- a page's `id` set settles it (exactly one match the section, else the
-- file, rung 2); on any other target, or empty, it is dropped.
sectioned :: HtmlEnv -> String -> Maybe String -> Int -> Answer
sectioned env target frag rung = case frag of
  Just f
    | null f -> AFile target rung
    | ofLangs ["markdown"] target -> anchor (hMd env) target f
    | ofLangs ["html"] target -> ASection target (if hits f == 1 then Just (percentDecode f) else Nothing) 2
  _ -> AFile target rung
 where
  hits f = length (filter (== percentDecode f) (maybe [] dIds (M.lookup target (hDocs env))))

outOfScope :: Answer
outOfScope = AUnresolved OutOfScope
