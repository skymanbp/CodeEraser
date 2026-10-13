-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Envelope layer (contracts/VERSIONING.md §1): owns the protocol
-- version, the byte-length precheck, dispatch on @type@, and the
-- @error@ reply. Every failure becomes a well-formed error line —
-- never a hello-shaped rejection (the M0 defect this module fixes),
-- never a crash.
module CE.Protocol (internalError, maxLineBytes, proto, respond) where

import qualified CE.Arch as Arch
import qualified CE.Audit as Audit
import qualified CE.Candidates as Candidates
import qualified CE.Similar.Rank as Rank
import qualified CE.Similar.Bags as Bags
import qualified CE.Clone as Clone
import qualified CE.Docdup as Docdup
import qualified CE.Docdup.Pairs as DocPairs
import qualified CE.Document as Document
import qualified CE.Erase as Erase
import qualified CE.Flow as Flow
import qualified CE.FourClass as FourClass
import qualified CE.FourClass.Moves as Moves
import qualified CE.Graph as Graph
import qualified CE.Handshake as Handshake
import qualified CE.Merge as Merge
import qualified CE.Query as Query
import qualified CE.Resolve as Resolve
import CE.Protocol.Version (majorMatches, maxLineBytes, proto)
import qualified CE.Scan as Scan
import qualified CE.Similar as Similar
import qualified CE.Structure as Structure
import CE.Tables (tablesDigest)
import qualified CE.Tables as Tables
import qualified CE.Tombstone as Tombstone
import qualified CE.Trend as Trend
import qualified CE.Verdict as Verdict
import CE.Wire (Answered, Respond)
import CE.Wire.Families (FamilyName (..))
import qualified CE.Wire.Families as F
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Just the envelope: @type@ for dispatch, @id@ for echoing into
-- error replies, @proto@ for the per-message major check. Unknown
-- extra fields are ignored (§1).
data Envelope = Envelope
  { envType :: String
  , envId :: Maybe Value
  , envProto :: Maybe String
  }

instance FromJSON Envelope where
  parseJSON = withObject "Envelope" $ \o ->
    Envelope <$> o .: "type" <*> o .:? "id" <*> o .:? "proto"

-- | Answer one request line with one response line.
respond :: String -> B8.ByteString -> B8.ByteString
respond version line
  | B8.length line > maxLineBytes =
      errReply Nothing "too_large" "request line exceeds the byte ceiling"
  | otherwise = case eitherDecodeStrict line of
      Left e -> errReply (lenientId line) "bad_request" ("parse error: " <> e)
      Right env -> dispatch version env line

-- | When the ENVELOPE fails to decode (type missing or mistyped) the
-- line may still be a JSON object carrying an id — echo it, so a
-- shape typo does not read as L2-down client-side (the non-echoed-id
-- desync rule, VERSIONING.md §1; M5-close review LOW).
lenientId :: B8.ByteString -> Maybe Value
lenientId line = case decodeStrict line of
  Just (Object o) -> KM.lookup "id" o
  _ -> Nothing

-- | The Main loop's exception barrier reply ("never a crash",
-- structurally): id stays null — the computation died before any
-- field of it could be trusted.
internalError :: String -> B8.ByteString
internalError = errReply Nothing "internal"

-- | One row per judgment family — clone/1 landed with the T3 batch
-- (M5-3e), docdup/1 with the docdup batch (M5-3g), verdict/1 with
-- the score batch (M5-3i), each walking the declare-then-implement
-- path the goldens pin. Five identical case arms once inflated
-- dispatch to CC 19 (the M5-close warn repayment): the table IS the
-- dispatch — and since batch 9 P12 the hello's capability list is
-- DERIVED from it (famCap), so a capability declared without a
-- responder is unrepresentable, not merely undisciplined.
data Fam = Fam
  { famName :: FamilyName
  , famRespond :: Respond
  }

-- | The capability `<name>/<version>` and the request type
-- `<name>.request`, both read off the family's one name
-- (CE.Wire.Families, plan v2.33 step 11a).
famCap, famType :: Fam -> String
famCap f = nameWord (famName f) <> "/" <> show (nameVersion (famName f))
famType f = nameWord (famName f) <> ".request"

families :: [Fam]
families =
  [ Fam F.fourclass FourClass.respond
  , Fam F.graph Graph.respond
  , Fam F.clone Clone.respond
  , Fam F.docdup Docdup.respond
  , Fam F.verdict Verdict.respond
  , Fam F.scan Scan.respond
  , Fam F.structure Structure.respond
  , Fam F.trend Trend.respond
  , Fam F.erase Erase.respond
  , Fam F.audit Audit.respond
  , Fam F.tombstone Tombstone.respond
  , Fam F.similar Similar.respond
  , Fam F.query Query.respond
  , Fam F.flow Flow.respond
  , Fam F.merge Merge.respond
  , Fam F.arch Arch.respond
  , Fam F.tables Tables.respond
  , Fam F.document Document.respond
  , Fam F.resolve Resolve.respond
  , Fam F.candidates Candidates.respond
  , Fam F.rank Rank.respond
  , Fam F.docpairs DocPairs.respond
  , Fam F.moves Moves.respond
  , Fam F.bags Bags.respond
  ]

-- | Every non-hello message must carry a proto with the server's
-- major (§1: 必带信封字段; 1.0.0 attack-review fix — the 0.x
-- implementation negotiated only at hello, so a skewed or bare
-- request was silently answered). hello keeps its own negotiation
-- reply (accept:false), which is richer than an error line.
dispatch :: String -> Envelope -> B8.ByteString -> B8.ByteString
dispatch version env line
  | envType env == "hello" =
      Handshake.respond proto ("hello" : map famCap families) tablesDigest version line
  | not (majorMatches (envProto env)) =
      errReply (envId env) "bad_request" ("proto missing or major-mismatched (server " <> proto <> ")")
  | (f : _) <- [f | f <- families, famType f == envType env] =
      familyReply env (famRespond f proto line)
  | otherwise = errReply (envId env) "unknown_type" ("unsupported type: " <> envType env)

-- | A family-level decode failure hands back rid = Nothing, but the
-- ENVELOPE id is already in hand here — falling back to it keeps a
-- single malformed request from desyncing the whole session (the
-- client treats a non-echoed id as L2-down, VERSIONING.md §1; Opus
-- review, one fix for every family).
familyReply :: Envelope -> Answered -> B8.ByteString
familyReply env (Left (rid, code, message)) = errReply (rid <|> envId env) code message
familyReply _ (Right bytes) = bytes

errReply :: Maybe Value -> String -> String -> B8.ByteString
errReply requestId code message =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("error" :: String)
      , "id" .= maybe Null id requestId
      , "code" .= code
      , "message" .= message
      ]
