-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Step 5's ten report families (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5): scan, dedup, clone and its
-- unit listing, docdup, the erase plan and its trail, churn, trend and
-- similar. The four legs every family takes (DocumentHarness), plus the
-- ones only these hold: each family's own refusals by name, dedup's
-- twelve summary counters, trend's verdict words in both languages,
-- the erase names read from CE.Erase.Cost, the trail's UTC stamp read
-- the way the measuring side writes it, and seeded documents that reach
-- every branch. The console form of these families — holes against
-- references, every catalogue key, the veto restated — rides in
-- DocumentProps5 with every other family's.
module DocumentProps6 (battery) where

import CE.Document (catalogue, respond)
import CE.Document.Contract (DocFamily (..), DocReq, Spec (..), Table (..))
import CE.Erase.Cost (classNames, reasonNames)
import CE.Erase.TrailLines (utcStamp)
import CE.Text (Line (..), Piece (..), langOf)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy.Char8 as BL
import Data.List (isPrefixOf, sort)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)
import DocumentGen (docRequest)
import DocumentGen6 (requests6)
import DocumentHarness (at, emptiesHeld, familiesNamed, fieldsHeld, int, items, judgedBy, path, refsHeld, sameBytes)
import WireHarness (refusedBy, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    (lines legNames)
    [emptiesHeld ten, fieldsHeld fieldTable ten, refsHeld judged && length judged == 2000, sameBytes (take 100 judged), refusals, twelve, verdictWords, eraseNames, stamped, seeded, pretty]

-- | One leg name per line, in the legs' order.
legNames :: String
legNames =
  "every step-5 empty document is the catalogue's\n\
  \every step-5 empty document has the fields its statement names\n\
  \every reference in 200 documents per step-5 family is a stated class inside its ranges\n\
  \the same step-5 request assembles to the same bytes\n\
  \the step-5 contracts refuse by name\n\
  \dedup's summary holds the twelve counters\n\
  \trend's verdict words are the face's, in both languages\n\
  \the erase plan and trail spell CE.Erase.Cost's names\n\
  \the trail's stamp is the measuring side's civil date\n\
  \the seeded step-5 documents are not vacuous\n\
  \the catalogue says pretty for scan and dedup alone"

ten :: [DocFamily]
ten = familiesNamed (words "scan dedup clone clone-units docdup erase erase-trail churn trend similar")

judged :: [(DocFamily, Value, Value)]
judged = judgedBy requests6 ten

-- | Each family's top-level fields in its empty document, `family key
-- type` (`ref`: a reference; `?`: or null).
fieldTable :: String
fieldTable =
  "scan schema string\nscan files array\nscan findings array\nscan summary object\nscan failed array\n\
  \dedup schema string\ndedup blocks array\ndedup groups array\ndedup summary object\n\
  \clone schema string\nclone clones array\nclone counts object\nclone-units schema string\nclone-units units array\n\
  \docdup schema string\ndocdup dups array\ndocdup counts object\n\
  \erase schema string\nerase rows array\nerase counts object\nerase families object\n\
  \erase-trail schema string\nerase-trail log string\nerase-trail present bool\nerase-trail rows array\n\
  \erase-trail unreadable array\nerase-trail counts object\n\
  \churn schema string\nchurn commits number\nchurn append_lines number\nchurn rewrite_lines number\n\
  \churn added_in_window number\nchurn surviving number\nchurn churned number\nchurn cochange array\n\
  \churn skipped_large_commits number\nchurn submodules_without_file_history array\n\
  \trend schema string\ntrend window number\ntrend pending number\ntrend rows array\ntrend failed array\ntrend judgment object\n\
  \similar schema string\nsimilar similar_rev number\nsimilar query object\nsimilar candidates array\nsimilar counts object\nsimilar degraded ref?"

-- | A request of a family: every stated range, table and fact, the
-- given ones as given and the rest empty or zero.
req :: String -> [(String, Integer)] -> [(String, [[Integer]])] -> [(String, Integer)] -> Value
req fam ranges ts fs = docRequest fam [(r, look r ranges 0) | r <- spRanges sp] [(tName t, look (tName t) ts []) | t <- spTables sp] [(n, look n fs 0) | (n, _) <- spFacts sp] Nothing
 where
  sp = maybe (Spec [] [] [] [] []) dfSpec (lookup fam [(dfName f, f) | f <- ten])
  look k xs d = fromMaybe d (lookup k xs)

-- | The named refusals, one per line: the family; its ranges, rows and
-- facts as JSON objects (what a line leaves out is empty or zero); the
-- degraded index (`-`: none); the refusal.
refusalTable :: String
refusalTable =
  "scan\t{\"files\":1,\"fns\":2,\"rows\":15}\t{\"files\":[[0,9,0,3]],\"fns\":[[0,0,1,2,2,0,1,1,0,0],[1,0,3,4,2,0,1,1,0,0]],\"levels\":[[0,9,0]],\"grades\":[[0,300,750],[1,50,75]]}\t{}\t-\tranges: rows is not files + 6 x fns\n\
  \scan\t{\"files\":1,\"fns\":2,\"rows\":13}\t{\"files\":[[0,9,0,3]],\"fns\":[[0,0,1,2,2,0,1,1,0,0],[1,0,3,4,2,0,1,1,0,0]],\"levels\":[[3,1,0],[1,1,0]],\"grades\":[[0,300,750],[1,50,75]]}\t{}\t-\tlevels: rows not strictly ascending\n\
  \scan\t{\"files\":1,\"fns\":0,\"rows\":1}\t{\"files\":[[0,1,0,99]]}\t{}\t-\tfiles 0: no language 99\n\
  \dedup\t{\"paths\":1,\"groups\":0}\t{}\t{\"fail\":1}\t-\tfacts: fail without check\n\
  \dedup\t{\"paths\":1,\"groups\":2}\t{\"groups\":[[0,1,50],[1,1,50]],\"members\":[[1,0,1,2],[0,0,3,4]]}\t{}\t-\tmembers: not in group order\n\
  \clone\t{\"units\":2}\t{\"pairs\":[[0,1,3,30,30,2]]}\t{}\t-\tpairs 0: column 5 is not a code 0..1\n\
  \clone-units\t{\"paths\":1,\"units\":2}\t{\"units\":[[1,0,0,30]]}\t{}\t-\tunits: not one row per slot in order\n\
  \docdup\t{\"paths\":1,\"segs\":1}\t{\"segs\":[[0,0,1,5,9]]}\t{}\t-\tsegs 0: column 4 is not a code 0..4\n\
  \erase\t{\"paths\":1,\"cands\":1}\t{\"cands\":[[0,0,0,0,0,0,7,1,0,1]]}\t{}\t-\tcands 0: column 0 is not a code 1..3\n\
  \erase\t{\"paths\":1,\"cands\":0}\t{\"outOfClass\":[[1,4]]}\t{}\t-\toutOfClass 0: column 0 is not a code 0..0\n\
  \erase\t{\"paths\":1,\"cands\":0}\t{}\t{\"applied\":2}\t-\tfacts: applied without apply\n\
  \erase-trail\t{\"paths\":1,\"records\":0,\"unread\":1}\t{\"unreadable\":[[0,3]]}\t{}\t-\trecords: a trail that is not present\n\
  \churn\t{\"paths\":1,\"submodules\":0}\t{\"cochange\":[[0,1,2]]}\t{}\t-\tcochange 0: column 1 out of range paths\n\
  \trend\t{\"points\":1,\"failed\":0}\t{\"points\":[[0,1,900,1000,3]]}\t{}\t-\tpoints 0: an axis without its value\n\
  \trend\t{\"points\":2,\"failed\":0}\t{\"points\":[[0,1,900,1000],[1,2,800,1000]],\"cliff\":[[1,5],[1,6]]}\t{}\t-\tcliff: more than one row\n\
  \similar\t{\"seats\":1,\"why\":0}\t{\"candidates\":[[0,0,3,1,0,0,0,0,0,0,0,3]]}\t{}\t-\tcandidates 0: column 11 is not a code 0..2\n\
  \churn\t{\"paths\":0,\"submodules\":0}\t{}\t{}\t0\tdegraded: out of range why"

refusals :: Bool
refusals = length cases == 17 && all refused cases
 where
  cases = map (splitOn '\t') (lines refusalTable)
  refused c = case c of
    [fam, rs, ts, fs, why, msg] -> refusedBy respond (degrade why (req fam (obj rs) (obj ts) (obj fs))) msg
    _ -> False
  obj :: (FromJSON a) => String -> [(String, a)]
  obj s = maybe [] M.toList (decode (BL.pack s))
  degrade why r = if why == "-" then r else setKey "degraded" (toJSON (read why :: Int)) r
  splitOn ch t = case break (== ch) t of
    (a, _ : rest) -> a : splitOn ch rest
    (a, []) -> [a]

-- | Every catalogued family states its layout: indented for scan and
-- dedup, one line for every other.
pretty :: Bool
pretty = sort [(Key.toString k, p) | Object o <- [catalogue], (k, f) <- KM.toList o, let p = path ["pretty"] f, p /= Just (Bool False)] == [("dedup", Just (Bool True)), ("scan", Just (Bool True))]

-- | dedup's summary: the twelve counters the report names, no more.
twelve :: Bool
twelve = all held [d | (f, _, d) <- judged, dfName f == "dedup"]
 where
  held d = case path ["summary"] d of
    Just (Object o) -> sort (map Key.toString (KM.keys o)) == sort want
    _ -> False
  want = words "files refreshed removed blocks groups hot_chained stale_skipped low_diversity_suppressed kgram window min_tokens min_distinct"

-- | The verdict line opens with the face's word for each code, and for
-- none, in both languages.
verdictWords :: Bool
verdictWords = and [any (word `isPrefixOf`) (said lang v) | (v, en, zh) <- table, (lang, word) <- [(0, en), (1, zh)]]
 where
  table =
    [ (Just 0, "trend verdict: improving", "趋势判决：上行")
    , (Just 1, "trend verdict: flat", "趋势判决：持平")
    , (Just 2, "trend verdict: degrading", "趋势判决：恶化")
    , (Nothing, "trend verdict: unjudged (below minPoints)", "趋势判决：未判（低于最小点数）")
    ]
  said lang v = [t | Line _ (Piece t _ _) <- speak lang (trendReq v)]
  trendReq v = req "trend" [("points", 2), ("failed", 0)] [("points", [[0, 1, 900, 1000], [1, 2, 800, 1000]]), ("slope", [[-50] | v /= Nothing]), ("verdict", maybe [] (\c -> [[c]]) v), ("knobs", [[0, 3], [1, 0]])] [("window", 2), ("pending", 0), ("fail", 0)]

-- | A request's lines in a language, through the family's own reader.
speak :: Integer -> Value -> [Line]
speak lang r = case (fromJSON (setKey "lang" (toJSON lang) r) :: Result DocReq) of
  Success q -> concat [dfLines f (langOf (Just lang)) q | f <- ten, Just (toJSON (dfName f)) == at r "family"]
  _ -> []

-- | Every class and reason the plan and the trail print is CE.Erase.Cost's,
-- and the catalogue lists the two tables as they live there.
eraseNames :: Bool
eraseNames =
  path ["erase", "classes"] catalogue == Just (toJSON classNames)
    && path ["erase", "reasons"] catalogue == Just (toJSON reasonNames)
    && all spelledThere [(n, r) | (f, _, d) <- judged, dfName f `elem` ["erase", "erase-trail"], r <- items d "rows", n <- [dfName f]]
 where
  spelledThere (n, r) =
    at r "class" `elem` map (Just . toJSON) classNames
      && (n == "erase-trail" || at r "reason" `elem` map (Just . toJSON) reasonNames)

-- | The stamp of five instants, read off the measuring side's
-- `utc_stamp` (cli/src/erase/log.rs).
stamped :: Bool
stamped =
  map utcStamp [0, 951782400000, 1700000000123, 4102444799999, 86399999]
    == ["1970-01-01T00:00:00Z", "2000-02-29T00:00:00Z", "2023-11-14T22:13:20Z", "2099-12-31T23:59:59Z", "1970-01-01T23:59:59Z"]

-- | Each family reaches what the legs above read.
seeded :: Bool
seeded = all (\(fam, test) -> any (\(f, _, d) -> dfName f == fam && test d) judged) cases
 where
  cases =
    [ ("scan", \d -> any ((== Just "fail") . (`at` "level")) (items d "findings") && not (null (items d "failed")))
    , ("dedup", \d -> not (null (items d "blocks")) && any (not . null . (`items` "members")) (items d "groups"))
    , ("clone", not . null . (`items` "clones"))
    , ("clone-units", not . null . (`items` "units"))
    , ("docdup", not . null . (`items` "dups"))
    , ("erase", \d -> int (path ["counts", "eraseable"] d) > 0 && int (path ["counts", "advisory"] d) > 0)
    , ("erase-trail", \d -> not (null (items d "rows")) && not (null (items d "unreadable")))
    , ("churn", \d -> length (items d "cochange") > 20 && not (null (items d "submodules_without_file_history")))
    , ("trend", \d -> path ["judgment", "cliff"] d /= Just Null && not (null (items d "failed")))
    , ("similar", \d -> at d "degraded" /= Just Null && not (null (items d "candidates")))
    ]
