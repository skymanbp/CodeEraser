-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | document.request handler (plan v2.32 steps 3-4; design booklet
-- docs/reference/authority-track.md §5): the report documents of the
-- query, rules, flow, merge and architecture families (step 3) and of
-- check, structure, join, deadcode, mentions, sites and the graph
-- screen (step 4) and of scan, dedup, clone, docdup, erase, churn,
-- trend and similar (step 5), assembled here — the fields, their order, the
-- counts, the schema id and the degraded bit are the judge's
-- statement, and every face prints the one document. Not a judgment
-- family: it judges nothing, it lays out what a judgment already
-- answered. Its own family rather than a key on each family's reply,
-- because some judge in batches the measuring side joins (flow by
-- `rowCap`, merge by its two caps) and a document folded over the
-- batches on that side would leave the assembly there. Every
-- repository string is a reference (CE.Document.Contract); the
-- catalogue — each family's schema id and empty document, flow's kind
-- names and judged languages, the site kinds — rides in the definition
-- package (CE.Tables). The empty document is the family's statement
-- and the battery's anchor; the measuring side never binds it: a
-- judgment that did not happen is still asked here, with `degraded`
-- and the facts the measuring side kept, and a core out of reach
-- refuses the face by name. Step 5 adds the console form: the request
-- names a language (`lang`, 0 en / 1 zh), the reply carries the
-- family's `lines` in it — `[stream, text, ref…]`, every number and
-- product word written, a `{}` hole per reference — and its veto
-- (`exit: {fail}`); the guard and the audit faces, which have no
-- report document, are families of their own whose `document` is `{}`
-- (they stay out of the catalogue: there is no document to list).
module CE.Document (blankOf, catalogue, emptyOf, families, respond) where

import qualified CE.Arch.Document as Arch
import qualified CE.Arch.Lines as ArchLines
import qualified CE.Audit.Document as Audit
import qualified CE.Churn.Document as Churn
import qualified CE.Churn.Lines as ChurnLines
import qualified CE.Clone.Document as Clone
import qualified CE.Clone.Lines as CloneLines
import qualified CE.Dedup.Document as Dedup
import qualified CE.Dedup.Lines as DedupLines
import qualified CE.Docdup.Document as Docdup
import qualified CE.Docdup.Lines as DocdupLines
import CE.Document.Read hiding (fields)
import qualified CE.Erase.Document as Erase
import qualified CE.Erase.Lines as EraseLines
import qualified CE.Erase.TrailLines as TrailLines
import qualified CE.Flow.Document as Flow
import qualified CE.Flow.Lines as FlowLines
import qualified CE.Graph.Document as Deadcode
import qualified CE.Graph.Lines as DeadcodeLines
import qualified CE.Graph.Screen as Screen
import qualified CE.Graph.Sites as Sites
import qualified CE.Graph.SitesLines as SitesLines
import qualified CE.Guard.Document as Guard
import qualified CE.Join.Document as Join
import qualified CE.Join.Lines as JoinLines
import qualified CE.Mention.Document as Mentions
import qualified CE.Mention.Lines as MentionLines
import qualified CE.Merge.Document as Merge
import qualified CE.Merge.Lines as MergeLines
import qualified CE.Query.Document as Query
import qualified CE.Query.Lines as QueryLines
import qualified CE.Scan.Document as Scan
import qualified CE.Scan.Lines as ScanLines
import qualified CE.Score.Document as Check
import qualified CE.Score.Lines as CheckLines
import qualified CE.Similar.Document as Similar
import qualified CE.Similar.Lines as SimilarLines
import qualified CE.Structure.Document as Structure
import qualified CE.Structure.Lines as StructureLines
import qualified CE.Text.Arch as ArchText
import qualified CE.Text.Check as CheckText
import qualified CE.Text.Churn as ChurnText
import qualified CE.Text.Clone as CloneText
import qualified CE.Text.Deadcode as DeadcodeText
import qualified CE.Text.Dedup as DedupText
import qualified CE.Text.Docdup as DocdupText
import qualified CE.Text.Erase as EraseText
import qualified CE.Text.Flow as FlowText
import qualified CE.Text.Join as JoinText
import qualified CE.Text.Mentions as MentionText
import qualified CE.Text.Merge as MergeText
import qualified CE.Text.Query as QueryText
import qualified CE.Text.Scan as ScanText
import qualified CE.Text.Similar as SimilarText
import qualified CE.Text.Sites as SitesText
import qualified CE.Text.Structure as StructureText
import qualified CE.Text.Trend as TrendText
import qualified CE.Trend.Document as Trend
import qualified CE.Trend.Lines as TrendLines
import qualified CE.Wire as Wire
import Data.Aeson (encode, object, (.=))
import Data.Aeson.Key (fromString)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M

-- | The documents, by the name a request gives: step 3's five, then
-- step 4's seven, each with its console lines and veto (step 5; the
-- graph screen has no console), then step 5's ten report families —
-- scan, dedup, clone and its unit listing, docdup, the erase plan and
-- its trail, churn, trend and similar — then step 5's two sentence
-- families.
families :: [DocFamily]
families =
  [ spoken ArchText.catalogue ArchLines.lines' never Arch.doc
  , spoken QueryText.catalogue (QueryLines.lines' False) never Query.queryDoc
  , spoken QueryText.catalogue (QueryLines.lines' True) QueryLines.veto Query.rulesDoc
  , spoken FlowText.catalogue FlowLines.lines' FlowLines.veto Flow.doc
  , spoken MergeText.catalogue MergeLines.lines' never Merge.doc
  ]
    <> [ spoken CheckText.catalogue CheckLines.lines' CheckLines.veto Check.doc
       , spoken StructureText.catalogue StructureLines.lines' never Structure.doc
       , spoken JoinText.catalogue JoinLines.lines' never Join.doc
       , spoken DeadcodeText.catalogue DeadcodeLines.lines' DeadcodeLines.veto Deadcode.doc
       , spoken MentionText.catalogue MentionLines.lines' never Mentions.doc
       , spoken SitesText.catalogue SitesLines.lines' never Sites.doc
       , Screen.doc
       ]
    <> [ spoken ScanText.catalogue ScanLines.lines' ScanLines.veto Scan.doc
       , spoken DedupText.catalogue DedupLines.lines' DedupLines.veto Dedup.doc
       , spoken CloneText.catalogue CloneLines.lines' never Clone.doc
       , spoken CloneText.catalogue CloneLines.unitLines never Clone.unitsDoc
       , spoken DocdupText.catalogue DocdupLines.lines' DocdupLines.veto Docdup.doc
       , spoken EraseText.catalogue EraseLines.lines' EraseLines.veto Erase.doc
       , spoken EraseText.catalogue TrailLines.lines' TrailLines.veto Erase.trailDoc
       , spoken ChurnText.catalogue ChurnLines.lines' never Churn.doc
       , spoken TrendText.catalogue TrendLines.lines' TrendLines.veto Trend.doc
       , spoken SimilarText.catalogue SimilarLines.lines' never Similar.doc
       ]
    <> [Guard.doc, Audit.doc]
 where
  never _ _ = False

familyOf :: DocReq -> Maybe DocFamily
familyOf req = do
  name <- dFamily req
  lookup name [(dfName f, f) | f <- families]

-- | A family's document over a blank request: every range and fact
-- zero, no row, the reason the measuring side's first text — the
-- statement's anchor, never a document a face prints (a family that
-- states no `why` range takes no reason).
emptyOf :: DocFamily -> Value
emptyOf fam = dfAssemble fam (blankOf fam Nothing)

-- | The empty request a family's statement implies, in a language.
blankOf :: DocFamily -> Maybe Integer -> DocReq
blankOf fam lang =
  DocReq
    { dId = "empty"
    , dFamily = Just (dfName fam)
    , dRanges = Just (M.fromList [(r, 0) | r <- spRanges sp])
    , dRows = Just (M.fromList [(tName t, []) | t <- spTables sp])
    , dFacts = Just (M.fromList [(n, 0) | (n, _) <- spFacts sp])
    , dDegraded = if "why" `elem` spRanges sp then Just 0 else Nothing
    , dLang = lang
    }
 where
  sp = dfSpec fam

-- | The `document` key of the definition package: per family its
-- schema id, its empty document, whether its face prints it indented,
-- and what else it lists.
catalogue :: Value
catalogue =
  object
    [ fromString (dfName f) .= object (["schema" .= dfSchema f, "empty" .= emptyOf f, "pretty" .= dfPretty f] <> dfCatalogue f)
    | f <- families
    , not (null (dfSchema f))
    ]

-- | A family name first, then the language, then that family's
-- statement.
offences :: DocReq -> Maybe String
offences req = case (dFamily req, familyOf req) of
  (Nothing, _) -> Just "document: missing family"
  (Just name, Nothing) -> Just ("document: unknown family " <> name)
  (_, Just fam)
    | maybe False (`notElem` [0, 1]) (dLang req) -> Just "document: lang is not 0 or 1"
    | otherwise -> offence fam req

-- | Over the cap only once the family is known: a request naming no
-- family is refused by name first.
overCap :: DocReq -> Bool
overCap req = maybe False (const (totalRows req > docRowCap)) (familyOf req)

-- | The document.result object: the family's document, its console
-- lines in the request's language and its veto, and the rows it was
-- assembled from; past the cap the family's empty document and its
-- lines, the reason named.
answer :: String -> Bool -> DocReq -> B8.ByteString
answer proto degraded req = BL.toStrict (encode (object (fields <> ["reason" .= ("document_too_large" :: String) | degraded])))
 where
  judged f = if degraded then blankOf f (dLang req) else req
  document = maybe (object []) (\f -> dfAssemble f (judged f)) (familyOf req)
  lines' = maybe [] (\f -> map lineValue (dfLines f (langOf (dLang req)) (judged f))) (familyOf req)
  fails = maybe False (\f -> dfExit f (judged f)) (familyOf req)
  fields =
    [ "proto" .= proto
    , "type" .= ("document.result" :: String)
    , "id" .= dId req
    , "document" .= document
    , "lines" .= lines'
    , "exit" .= object ["fail" .= fails]
    , "counts" .= object ["rows" .= totalRows req]
    , "degraded" .= degraded
    ]

-- | decode → cap → contract → assemble.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = Wire.family "document" dId overCap offences (answer proto True) (answer proto False)
