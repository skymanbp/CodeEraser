{-# LANGUAGE DataKinds #-}
{-# LANGUAGE DerivingVia #-}
{-# LANGUAGE OverloadedStrings #-}

-- | The flow table's shapes (plan v2.32 step 1): the rows a flow table
-- is made of — design booklet docs/reference/analysis-track.md §5.1 —
-- with the measuring side's field names. Beside CE.Lang.Spec for the
-- reason the measuring side kept its rows apart from its table: the
-- row shapes alone fill a module. A position is a string in the
-- syntax the lowering resolves (`name`, `@kind`, `a/b`, `a|b`, `*`,
-- `.`, `""`); what each row means is documented at the data.
module CE.Lang.Spec.Flow
  ( Assign (..)
  , Case (..)
  , Catch (..)
  , Decl (..)
  , DefaultArm (..)
  , Fall (..)
  , FlowSpec (..)
  , If (..)
  , Loop (..)
  , Macro (..)
  , Mode (..)
  , Scope (..)
  , Switch (..)
  , Try (..)
  , With (..)
  ) where

import CE.Lang.Table (Blank (..), Row (..))
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import GHC.Generics (Generic)

type Kinds = [String]

type Pos = String

type Pairs = [(String, String)]

type Triples = [(String, Pos, Pos)]

type Ops = [(String, Pos, Kinds)]

type Marks = [(String, Scope)]

-- | Where a declaration is visible.
data Scope = ScopeBlock | ScopeFunction
  deriving stock (Eq, Generic)
  deriving (FromJSON, ToJSON) via Row 5 Scope

-- | What an assignment does to its target.
data Mode = ModeWrite | ModeReadWrite | ModeOuter
  deriving stock (Eq, Generic)
  deriving (FromJSON, ToJSON) via Row 4 Mode

-- | Whether an arm runs into the next one.
data Fall = FallNever | FallAlways | FallStatement
  deriving stock (Eq, Generic)
  deriving (FromJSON, ToJSON) via Row 4 Fall

instance Blank Scope where
  blank = ScopeBlock

instance Blank Mode where
  blank = ModeWrite

instance Blank Fall where
  blank = FallNever

-- | When an arm is its switch's default: never, by its kind, when
-- nothing sits at a position, or when a position holds a token.
data DefaultArm = ArmNever | ArmKind | ArmMissing Pos | ArmHolds Pos String
  deriving stock (Eq)

instance Blank DefaultArm where
  blank = ArmNever

instance ToJSON DefaultArm where
  toJSON ArmNever = "never"
  toJSON ArmKind = "kind"
  toJSON (ArmMissing p) = object ["missing" .= p]
  toJSON (ArmHolds p t) = object ["holds" .= (p, t)]

instance FromJSON DefaultArm where
  parseJSON (String "never") = pure ArmNever
  parseJSON (String "kind") = pure ArmKind
  parseJSON (Object o) = case KM.toList o of
    [("missing", p)] -> ArmMissing <$> parseJSON p
    [("holds", pt)] -> uncurry ArmHolds <$> parseJSON pt
    _ -> fail "a default arm is never, kind, {missing} or {holds}"
  parseJSON v = fail ("unknown default arm " <> show v)

data If = If {ifCond, ifThen, ifElse, ifInit :: Pos}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 If

data Loop = Loop
  { lpKind, lpHeader, lpBody, lpCond, lpInit, lpUpdate, lpTarget, lpIter :: Pos
  , lpMarker :: Marks
  , lpBodyFirst, lpUntil :: Bool
  , lpElse :: Pos
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Loop

data Switch = Switch
  { swKind, swSubject, swArms, swInit, swBinder :: Pos
  , swAlwaysDefault, swEmptyNoreturn, swPassesBreak :: Bool
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Switch

data Case = Case
  { caKind :: String
  , caDefault :: DefaultArm
  , caFallthrough :: Fall
  , caPattern, caValue, caGuard, caBody :: Pos
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Case

data Try = Try {trKind, trBody, trElse, trResources :: Pos}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Try

data Catch = Catch {ctKind, ctParam, ctValue, ctBody :: Pos}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Catch

data With = With {wiKind, wiItem, wiValue, wiBinder, wiBody :: Pos}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 With

data Decl = Decl
  { dcKind, dcToken :: String
  , dcScope :: Scope
  , dcItems, dcPair, dcBinder, dcInit, dcStorage, dcAlternative :: Pos
  , dcRedeclare :: Bool
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Decl

data Assign = Assign {asKind, asOp, asLeft, asRight :: Pos, asMode :: Mode}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Assign

data Macro = Macro {mcKind, mcName, mcArgs :: Pos, mcStrings :: Kinds}
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 Macro

-- | One language's flow table: the §5.3 rows as its grammar spells them.
data FlowSpec = FlowSpec
  { flBlockKinds, flSpliceKinds, flWrapperKinds, flEmptyKinds :: Kinds
  , flIf :: If
  , flElifKinds :: Kinds
  , flElseKinds :: Pairs
  , flCondWrappers :: Triples
  , flLoops :: [Loop]
  , flSwitches :: [Switch]
  , flCases :: [Case]
  , flTries :: [Try]
  , flCatches :: [Catch]
  , flFinallyKinds :: Pairs
  , flWiths :: [With]
  , flReturnKinds, flThrowKinds, flBreakKinds, flContinueKinds :: Kinds
  , flGotos :: Pairs
  , flLabels :: Triples
  , flSelfLabel :: Pos
  , flYieldKinds, flFallthroughKinds, flNoreturn :: Kinds
  , flNoreturnAttrs :: Pairs
  , flReturnCalls :: Kinds
  , flCallForms :: Pairs
  , flMacros :: [Macro]
  , flConstTrue, flConstFalse :: Pairs
  , flIntKinds, flDynamicNames, flDynamicKinds :: Kinds
  , flScoping :: Scope
  , flFirstWriteDeclares :: Bool
  , flParams :: Pairs
  , flReceiverNames :: Kinds
  , flResults :: Pos
  , flDecls :: [Decl]
  , flLastingStorage, flPatternKinds, flPatternIdents, flDottedPatterns :: Kinds
  , flUpperPatternPaths :: Bool
  , flBinderPaths :: Pairs
  , flPrototypeKinds, flPrototypeReads :: Kinds
  , flPatternBinders :: Pairs
  , flBranchBinders, flNonlocalKinds, flLocalOnlyScopes, flIdentKinds :: Kinds
  , flNamePositions :: Pairs
  , flShorthandKinds, flInterpolatedStrings, flTypeReads :: Kinds
  , flFieldParams :: Pairs
  , flHeadReads, flDispatchCalls, flMemberWriteBases :: Kinds
  , flAssigns :: [Assign]
  , flUpdateKinds :: Pairs
  , flConditionalCtx :: Ops
  , flDefaultArgFields :: Pairs
  , flCaptureKinds :: Kinds
  , flForwardCaptures :: Bool
  , flAddressOps :: Pairs
  , flRefBindingKinds, flDiscardNames :: Kinds
  }
  deriving stock (Eq, Generic)
  deriving (Blank, FromJSON, ToJSON) via Row 2 FlowSpec
