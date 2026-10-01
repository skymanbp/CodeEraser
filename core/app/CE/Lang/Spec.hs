{-# LANGUAGE DataKinds #-}
{-# LANGUAGE DerivingVia #-}
{-# LANGUAGE OverloadedStrings #-}

-- | The shapes of the definition tables (plan v2.32 step 1; design
-- booklet docs/reference/authority-track.md §4.2): one record per
-- table, its JSON field names the measuring side's own (`fn_kinds`,
-- `name_fields`…), so the package `tables/1` answers lands in the
-- structs that read it today. The flow table's shapes sit beside it in
-- CE.Lang.Spec.Flow. A field's meaning is documented once, at the data
-- (CE.Lang.<Language>), never restated here.
module CE.Lang.Spec
  ( CallSite (..)
  , Kinds
  , LangTables (..)
  , Larger (..)
  , Language (..)
  , NameStyle (..)
  , Overloads (..)
  , Pairs
  , ScanSpec (..)
  , SiteKind (..)
  , SlotSpec (..)
  , Specifier (..)
  ) where

import CE.Lang.Spec.Flow (FlowSpec)
import CE.Lang.Table (Blank (..), Row (..))
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (Parser)
import GHC.Generics (Generic, Generically (..))

-- | Node kinds, tokens or names, as the field says.
type Kinds = [String]

-- | A kind and one string about it (a field, mostly).
type Pairs = [(String, String)]

-- | The readability check's function-name convention.
data NameStyle = StyleSnake | StyleMixedCaps | StyleAny
  deriving (Eq)

instance Blank NameStyle where
  blank = StyleAny

instance ToJSON NameStyle where
  toJSON StyleSnake = "snake"
  toJSON StyleMixedCaps = "mixed_caps"
  toJSON StyleAny = "any"

instance FromJSON NameStyle where
  parseJSON v = case [s | s <- [StyleSnake, StyleMixedCaps, StyleAny], toJSON s == v] of
    s : _ -> pure s
    [] -> fail ("unknown name style " <> show v)

-- | How same-named callables of one scope are told apart (C++, Java).
data Overloads = Overloads {ovOptional, ovVariadic, ovIgnored, ovSpread, ovUnreachable :: Kinds}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Overloads

-- | A language's scan table: units, both complexity metrics, the
-- literal delimiters, the call arcs.
data ScanSpec = ScanSpec
  { scFnKinds, scParamListKinds, scCcKinds, scCcOperators, scChainKinds :: Kinds
  , scCocNestingKinds, scIfKinds, scCocFlatKinds, scCocNestOnlyKinds :: Kinds
  , scCocOperators, scCocJumpKinds, scLabelKinds, scCommentKinds, scLiteralDelims :: Kinds
  , scCallKinds, scCallNameKinds, scCallMemberKinds, scCallSelfWords :: Kinds
  , scCallMemberScopes, scOwnerKinds, scCallImportKinds :: Kinds
  , scFnRequiredFields, scOpaqueFields :: Pairs
  , scNameStyle :: NameStyle
  , scCallFields :: (String, Maybe String)
  , scOverloads :: Maybe Overloads
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 ScanSpec

-- | A language's merge-family position classes; pieces join list by
-- list (TypeScript and TSX share one, C and C++ another), the helper
-- lines by the larger.
data SlotSpec = SlotSpec
  { slExprKinds, slTypeKinds, slOtherKinds, slStmtKinds, slContainerKinds :: Kinds
  , slTargetLists, slPartKinds :: Kinds
  , slNameFields, slTargetFields, slPartFields :: Pairs
  , slTargetOps :: [(String, String, String)]
  , slHelperLines :: Larger
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 SlotSpec
  deriving (Semigroup, Monoid) via Generically SlotSpec

-- | A count two pieces join by taking the larger (a table's helper
-- lines: a piece that does not state them states none).
newtype Larger = Larger Int
  deriving stock (Eq)
  deriving (Blank, FromJSON, ToJSON) via Int

instance Semigroup Larger where
  Larger a <> Larger b = Larger (max a b)

instance Monoid Larger where
  mempty = Larger 0

-- | Where a site's specifier string comes from. Written adjacently
-- tagged — `{"form": "field", "arg": "source"}`, a form with no
-- argument without `arg` — the shape VERSIONING 7.7.0 fixes.
data Specifier
  = Field String
  | NameIfNoBody
  | EachImportTarget
  | FieldIfStar String
  | Literal String
  | FirstNamed Bool
  | UseTargets
  | Spanned String String
  deriving (Eq)

instance ToJSON Specifier where
  toJSON spec = object (("form" .= form) : ["arg" .= a | Just a <- [arg]])
   where
    (form, arg) = case spec of
      Field f -> ("field" :: String, Just (toJSON f))
      NameIfNoBody -> ("name_if_no_body", Nothing)
      EachImportTarget -> ("each_import_target", Nothing)
      FieldIfStar f -> ("field_if_star", Just (toJSON f))
      Literal l -> ("literal", Just (toJSON l))
      FirstNamed star -> ("first_named", Just (object ["star" .= star]))
      UseTargets -> ("use_targets", Nothing)
      Spanned from to -> ("spanned", Just (object ["from" .= from, "to" .= to]))

instance FromJSON Specifier where
  parseJSON = withObject "specifier" $ \o -> do
    form <- o .: "form"
    let arg :: (FromJSON a) => Parser a
        arg = o .: "arg"
    case form :: String of
      "field" -> Field <$> arg
      "name_if_no_body" -> pure NameIfNoBody
      "each_import_target" -> pure EachImportTarget
      "field_if_star" -> FieldIfStar <$> arg
      "literal" -> Literal <$> arg
      "first_named" -> arg >>= withObject "first_named" (fmap FirstNamed . (.: "star"))
      "use_targets" -> pure UseTargets
      "spanned" -> arg >>= withObject "spanned" (\a -> Spanned <$> a .: "from" <*> a .: "to")
      _ -> fail ("unknown specifier form " <> form)

-- | (node kind, frozen label, specifier source): one way a site opens.
data SiteKind = SiteKind {skNode :: String, skLabel :: String, skVia :: Specifier}
  deriving (Eq)

instance ToJSON SiteKind where
  toJSON s = object ["node" .= skNode s, "label" .= skLabel s, "via" .= skVia s]

instance FromJSON SiteKind where
  parseJSON = withObject "site" $ \o -> SiteKind <$> o .: "node" <*> o .: "label" <*> o .: "via"

-- | A call naming its target by an argument (Lua, R).
data CallSite = CallSite {csLabel, csPackage :: String, csCallees :: Kinds, csUnquoted :: Maybe String}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 CallSite

-- | One row of the language table: the frozen wire code, the report
-- name, the extensions, and the arms it belongs to.
data Language = Language
  { lgCode :: Int
  , lgName :: String
  , lgScanOnly, lgProseOnly, lgJudged, lgDocument :: Bool
  , lgExts :: Kinds
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Language

-- | Every table of one judged language.
data LangTables = LangTables
  { ltName :: String
  , ltScan :: ScanSpec
  , ltFlow :: Maybe FlowSpec
  , ltSlot :: Maybe SlotSpec
  , ltSites :: [SiteKind]
  , ltCalls :: [CallSite]
  , ltProtected :: [(String, Int)]
  , ltExtra :: Kinds
  , ltDocstringHosts :: Kinds
  }

-- | A language's document: `name` and its top-level lists, the `[scan]`
-- and `[flow]` tables, and the `[[slot]]` pieces joined list by list.
-- An absent key is the empty table (no flow or slot table at all), an
-- unknown one refuses the document.
instance FromJSON LangTables where
  parseJSON = withObject "language" $ \o -> do
    case filter (`notElem` keys) (KM.keys o) of
      k : _ -> fail ("unknown key " <> show k)
      [] -> pure ()
    slots <- o .:? "slot" .!= []
    LangTables
      <$> o .: "name"
      <*> o .:? "scan" .!= blank
      <*> o .:? "flow"
      <*> pure (if null slots then Nothing else Just (mconcat slots))
      <*> o .:? "sites" .!= []
      <*> o .:? "calls" .!= []
      <*> o .:? "protected" .!= []
      <*> o .:? "extra" .!= []
      <*> o .:? "docstring_hosts" .!= []
   where
    keys = ["name", "scan", "flow", "slot", "sites", "calls", "protected", "extra", "docstring_hosts"]
