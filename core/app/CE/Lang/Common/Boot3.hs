-- | The GHC global package table, part 3 of 3 (plan v2.32 step 1),
-- transcribed from cli/src/graph/ladder/hs_boot.rs at a378e78c; from this
-- commit on the core is the authority.
module CE.Lang.Common.Boot3 where

boot :: String
boot =
  "ghc-internal\n\
  \  GHC.Internal.AllocationLimitHandler GHC.Internal.Arr GHC.Internal.ArrayArray\n\
  \  GHC.Internal.Base GHC.Internal.Bignum.Backend GHC.Internal.Bignum.Backend.Native\n\
  \  GHC.Internal.Bignum.Backend.Selected GHC.Internal.Bignum.BigNat\n\
  \  GHC.Internal.Bignum.Integer GHC.Internal.Bignum.Natural GHC.Internal.Bignum.Primitives\n\
  \  GHC.Internal.Bignum.WordArray GHC.Internal.Bits GHC.Internal.ByteOrder\n\
  \  GHC.Internal.CString GHC.Internal.Char GHC.Internal.Classes GHC.Internal.Clock\n\
  \  GHC.Internal.ClosureTypes GHC.Internal.Conc.Bound GHC.Internal.Conc.IO\n\
  \  GHC.Internal.Conc.POSIX GHC.Internal.Conc.POSIX.Const GHC.Internal.Conc.Signal\n\
  \  GHC.Internal.Conc.Sync GHC.Internal.Conc.Windows GHC.Internal.ConsoleHandler\n\
  \  GHC.Internal.Control.Arrow GHC.Internal.Control.Category\n\
  \  GHC.Internal.Control.Concurrent.MVar GHC.Internal.Control.Exception\n\
  \  GHC.Internal.Control.Exception.Base GHC.Internal.Control.Monad\n\
  \  GHC.Internal.Control.Monad.Fail GHC.Internal.Control.Monad.Fix\n\
  \  GHC.Internal.Control.Monad.IO.Class GHC.Internal.Control.Monad.ST\n\
  \  GHC.Internal.Control.Monad.ST.Imp GHC.Internal.Control.Monad.ST.Lazy\n\
  \  GHC.Internal.Control.Monad.ST.Lazy.Imp GHC.Internal.Control.Monad.Zip\n\
  \  GHC.Internal.Data.Bits GHC.Internal.Data.Bool GHC.Internal.Data.Coerce\n\
  \  GHC.Internal.Data.Data GHC.Internal.Data.Dynamic GHC.Internal.Data.Either\n\
  \  GHC.Internal.Data.Eq GHC.Internal.Data.Foldable GHC.Internal.Data.Function\n\
  \  GHC.Internal.Data.Functor GHC.Internal.Data.Functor.Const\n\
  \  GHC.Internal.Data.Functor.Identity GHC.Internal.Data.Functor.Utils\n\
  \  GHC.Internal.Data.IORef GHC.Internal.Data.Ix GHC.Internal.Data.List\n\
  \  GHC.Internal.Data.List.NonEmpty GHC.Internal.Data.Maybe GHC.Internal.Data.Monoid\n\
  \  GHC.Internal.Data.NonEmpty GHC.Internal.Data.OldList GHC.Internal.Data.Ord\n\
  \  GHC.Internal.Data.Proxy GHC.Internal.Data.STRef GHC.Internal.Data.STRef.Strict\n\
  \  GHC.Internal.Data.Semigroup.Internal GHC.Internal.Data.String\n\
  \  GHC.Internal.Data.Traversable GHC.Internal.Data.Tuple GHC.Internal.Data.Type.Bool\n\
  \  GHC.Internal.Data.Type.Coercion GHC.Internal.Data.Type.Equality\n\
  \  GHC.Internal.Data.Type.Ord GHC.Internal.Data.Typeable GHC.Internal.Data.Unique\n\
  \  GHC.Internal.Data.Version GHC.Internal.Data.Void GHC.Internal.Debug\n\
  \  GHC.Internal.Debug.Trace GHC.Internal.Desugar GHC.Internal.Encoding.UTF8\n\
  \  GHC.Internal.Enum GHC.Internal.Environment GHC.Internal.Err GHC.Internal.Event.TimeOut\n\
  \  GHC.Internal.Event.Windows GHC.Internal.Event.Windows.Clock\n\
  \  GHC.Internal.Event.Windows.ConsoleEvent GHC.Internal.Event.Windows.FFI\n\
  \  GHC.Internal.Event.Windows.ManagedThreadPool GHC.Internal.Event.Windows.Thread\n\
  \  GHC.Internal.Exception GHC.Internal.Exception.Backtrace GHC.Internal.Exception.Context\n\
  \  GHC.Internal.Exception.Type GHC.Internal.ExecutionStack\n\
  \  GHC.Internal.ExecutionStack.Internal GHC.Internal.Exts GHC.Internal.Fingerprint\n\
  \  GHC.Internal.Fingerprint.Type GHC.Internal.Float GHC.Internal.Float.ConversionUtils\n\
  \  GHC.Internal.Float.RealFracMethods GHC.Internal.Foreign.C.ConstPtr\n\
  \  GHC.Internal.Foreign.C.Error GHC.Internal.Foreign.C.String\n"

boot2 :: String
boot2 =
  "  GHC.Internal.Foreign.C.String.Encoding GHC.Internal.Foreign.C.Types\n\
  \  GHC.Internal.Foreign.Concurrent GHC.Internal.Foreign.ForeignPtr\n\
  \  GHC.Internal.Foreign.ForeignPtr.Imp GHC.Internal.Foreign.ForeignPtr.Unsafe\n\
  \  GHC.Internal.Foreign.Marshal.Alloc GHC.Internal.Foreign.Marshal.Array\n\
  \  GHC.Internal.Foreign.Marshal.Error GHC.Internal.Foreign.Marshal.Pool\n\
  \  GHC.Internal.Foreign.Marshal.Safe GHC.Internal.Foreign.Marshal.Unsafe\n\
  \  GHC.Internal.Foreign.Marshal.Utils GHC.Internal.Foreign.Ptr\n\
  \  GHC.Internal.Foreign.StablePtr GHC.Internal.Foreign.Storable GHC.Internal.ForeignPtr\n\
  \  GHC.Internal.ForeignSrcLang GHC.Internal.Functor.ZipList GHC.Internal.GHCi\n\
  \  GHC.Internal.GHCi.Helpers GHC.Internal.Generics GHC.Internal.Heap.Closures\n\
  \  GHC.Internal.Heap.Constants GHC.Internal.Heap.InfoTable\n\
  \  GHC.Internal.Heap.InfoTable.Types GHC.Internal.Heap.InfoTableProf\n\
  \  GHC.Internal.Heap.ProfInfo.Types GHC.Internal.IO GHC.Internal.IO.Buffer\n\
  \  GHC.Internal.IO.BufferedIO GHC.Internal.IO.Device GHC.Internal.IO.Encoding\n\
  \  GHC.Internal.IO.Encoding.CodePage GHC.Internal.IO.Encoding.CodePage.API\n\
  \  GHC.Internal.IO.Encoding.CodePage.Table GHC.Internal.IO.Encoding.Failure\n\
  \  GHC.Internal.IO.Encoding.Iconv GHC.Internal.IO.Encoding.Latin1\n\
  \  GHC.Internal.IO.Encoding.Types GHC.Internal.IO.Encoding.UTF16\n\
  \  GHC.Internal.IO.Encoding.UTF32 GHC.Internal.IO.Encoding.UTF8 GHC.Internal.IO.Exception\n\
  \  GHC.Internal.IO.FD GHC.Internal.IO.Handle GHC.Internal.IO.Handle.FD\n\
  \  GHC.Internal.IO.Handle.Internals GHC.Internal.IO.Handle.Lock\n\
  \  GHC.Internal.IO.Handle.Text GHC.Internal.IO.Handle.Types\n\
  \  GHC.Internal.IO.Handle.Windows GHC.Internal.IO.IOMode GHC.Internal.IO.StdHandles\n\
  \  GHC.Internal.IO.SubSystem GHC.Internal.IO.Unsafe GHC.Internal.IO.Windows.Encoding\n\
  \  GHC.Internal.IO.Windows.Handle GHC.Internal.IO.Windows.Paths GHC.Internal.IOArray\n\
  \  GHC.Internal.IORef GHC.Internal.InfoProv GHC.Internal.InfoProv.Types GHC.Internal.Int\n\
  \  GHC.Internal.Integer GHC.Internal.Integer.Logarithms GHC.Internal.IsList\n\
  \  GHC.Internal.Ix GHC.Internal.LanguageExtensions GHC.Internal.Lexeme GHC.Internal.List\n\
  \  GHC.Internal.MVar GHC.Internal.Magic GHC.Internal.Magic.Dict GHC.Internal.Maybe\n\
  \  GHC.Internal.Natural GHC.Internal.Num GHC.Internal.Numeric\n\
  \  GHC.Internal.Numeric.Natural GHC.Internal.OverloadedLabels GHC.Internal.Pack\n\
  \  GHC.Internal.Prim GHC.Internal.Prim.Exception GHC.Internal.Prim.Ext\n\
  \  GHC.Internal.Prim.Panic GHC.Internal.Prim.PtrEq GHC.Internal.PrimopWrappers\n\
  \  GHC.Internal.Profiling GHC.Internal.Ptr GHC.Internal.RTS.Flags\n\
  \  GHC.Internal.RTS.Flags.Test GHC.Internal.Read GHC.Internal.Real GHC.Internal.Records\n\
  \  GHC.Internal.ResponseFile GHC.Internal.ST GHC.Internal.STRef GHC.Internal.Show\n\
  \  GHC.Internal.Stable GHC.Internal.StableName GHC.Internal.Stack\n\
  \  GHC.Internal.Stack.Annotation GHC.Internal.Stack.CCS GHC.Internal.Stack.CloneStack\n\
  \  GHC.Internal.Stack.Constants GHC.Internal.Stack.ConstantsProf\n\
  \  GHC.Internal.Stack.Decode GHC.Internal.Stack.Types GHC.Internal.StaticPtr\n\
  \  GHC.Internal.Stats GHC.Internal.Storable GHC.Internal.System.Environment\n\
  \  GHC.Internal.System.Environment.Blank GHC.Internal.System.Exit GHC.Internal.System.IO\n"

boot3 :: String
boot3 =
  "  GHC.Internal.System.IO.Error GHC.Internal.System.Mem\n\
  \  GHC.Internal.System.Mem.StableName GHC.Internal.System.Posix.Internals\n\
  \  GHC.Internal.System.Posix.Types GHC.Internal.TH.Lib GHC.Internal.TH.Lift\n\
  \  GHC.Internal.TH.Quote GHC.Internal.TH.Syntax GHC.Internal.Text.ParserCombinators.ReadP\n\
  \  GHC.Internal.Text.ParserCombinators.ReadPrec GHC.Internal.Text.Read\n\
  \  GHC.Internal.Text.Read.Lex GHC.Internal.Text.Show GHC.Internal.TopHandler\n\
  \  GHC.Internal.Tuple GHC.Internal.Type.Reflection GHC.Internal.Type.Reflection.Unsafe\n\
  \  GHC.Internal.TypeError GHC.Internal.TypeLits GHC.Internal.TypeLits.Internal\n\
  \  GHC.Internal.TypeNats GHC.Internal.TypeNats.Internal GHC.Internal.Types\n\
  \  GHC.Internal.Unicode GHC.Internal.Unsafe.Coerce GHC.Internal.Weak\n\
  \  GHC.Internal.Weak.Finalize GHC.Internal.Windows GHC.Internal.Word\n\
  \ghc-platform GHC.Platform.ArchOS\n\
  \ghc-prim\n\
  \  GHC.CString GHC.Classes GHC.Debug GHC.Magic GHC.Magic.Dict GHC.Prim GHC.Prim.Exception\n\
  \  GHC.Prim.Ext GHC.Prim.Panic GHC.Prim.PtrEq GHC.PrimopWrappers GHC.Tuple GHC.Types\n\
  \ghc-toolchain\n\
  \  GHC.Toolchain GHC.Toolchain.CheckArm GHC.Toolchain.Lens GHC.Toolchain.Monad\n\
  \  GHC.Toolchain.NormaliseTriple GHC.Toolchain.ParseTriple GHC.Toolchain.PlatformDetails\n\
  \  GHC.Toolchain.Prelude GHC.Toolchain.Program GHC.Toolchain.Target\n\
  \  GHC.Toolchain.Tools.Ar GHC.Toolchain.Tools.Cc GHC.Toolchain.Tools.Cpp\n\
  \  GHC.Toolchain.Tools.Cxx GHC.Toolchain.Tools.Link GHC.Toolchain.Tools.MergeObjs\n\
  \  GHC.Toolchain.Tools.Nm GHC.Toolchain.Tools.Ranlib GHC.Toolchain.Tools.Readelf\n\
  \  GHC.Toolchain.Utils\n\
  \ghci\n\
  \  GHCi.BinaryArray GHCi.BreakArray GHCi.CreateBCO GHCi.Debugger GHCi.FFI GHCi.InfoTable\n\
  \  GHCi.Message GHCi.ObjLink GHCi.RemoteTypes GHCi.ResolvedBCO GHCi.Run GHCi.Server\n\
  \  GHCi.Signals GHCi.StaticPtrTable GHCi.TH GHCi.TH.Binary GHCi.Utils\n\
  \haddock-api Documentation.Haddock\n\
  \haddock-library\n\
  \  Documentation.Haddock.Doc Documentation.Haddock.Markup Documentation.Haddock.Parser\n\
  \  Documentation.Haddock.Types\n\
  \haskeline\n\
  \  System.Console.Haskeline System.Console.Haskeline.Completion\n\
  \  System.Console.Haskeline.History System.Console.Haskeline.IO\n\
  \  System.Console.Haskeline.Internal\n\
  \hpc Trace.Hpc.Mix Trace.Hpc.Reflect Trace.Hpc.Tix Trace.Hpc.Util\n\
  \integer-gmp GHC.Integer.GMP.Internals\n"

boot4 :: String
boot4 =
  "mtl\n\
  \  Control.Monad.Accum Control.Monad.Cont Control.Monad.Cont.Class\n\
  \  Control.Monad.Error.Class Control.Monad.Except Control.Monad.Identity\n\
  \  Control.Monad.RWS Control.Monad.RWS.CPS Control.Monad.RWS.Class Control.Monad.RWS.Lazy\n\
  \  Control.Monad.RWS.Strict Control.Monad.Reader Control.Monad.Reader.Class\n\
  \  Control.Monad.Select Control.Monad.State Control.Monad.State.Class\n\
  \  Control.Monad.State.Lazy Control.Monad.State.Strict Control.Monad.Trans\n\
  \  Control.Monad.Writer Control.Monad.Writer.CPS Control.Monad.Writer.Class\n\
  \  Control.Monad.Writer.Lazy Control.Monad.Writer.Strict\n\
  \os-string\n\
  \  System.OsString System.OsString.Data.ByteString.Short\n\
  \  System.OsString.Data.ByteString.Short.Internal\n\
  \  System.OsString.Data.ByteString.Short.Word16 System.OsString.Encoding\n\
  \  System.OsString.Encoding.Internal System.OsString.Internal\n\
  \  System.OsString.Internal.Exception System.OsString.Internal.Types\n\
  \  System.OsString.Posix System.OsString.Windows\n\
  \parsec\n\
  \  Text.Parsec Text.Parsec.ByteString Text.Parsec.ByteString.Lazy Text.Parsec.Char\n\
  \  Text.Parsec.Combinator Text.Parsec.Error Text.Parsec.Expr Text.Parsec.Language\n\
  \  Text.Parsec.Perm Text.Parsec.Pos Text.Parsec.Prim Text.Parsec.String Text.Parsec.Text\n\
  \  Text.Parsec.Text.Lazy Text.Parsec.Token Text.ParserCombinators.Parsec\n\
  \  Text.ParserCombinators.Parsec.Char Text.ParserCombinators.Parsec.Combinator\n\
  \  Text.ParserCombinators.Parsec.Error Text.ParserCombinators.Parsec.Expr\n\
  \  Text.ParserCombinators.Parsec.Language Text.ParserCombinators.Parsec.Perm\n\
  \  Text.ParserCombinators.Parsec.Pos Text.ParserCombinators.Parsec.Prim\n\
  \  Text.ParserCombinators.Parsec.Token\n\
  \pretty\n\
  \  Text.PrettyPrint Text.PrettyPrint.Annotated Text.PrettyPrint.Annotated.HughesPJ\n\
  \  Text.PrettyPrint.Annotated.HughesPJClass Text.PrettyPrint.HughesPJ\n\
  \  Text.PrettyPrint.HughesPJClass\n\
  \process\n\
  \  System.Cmd System.Process System.Process.CommunicationHandle\n\
  \  System.Process.CommunicationHandle.Internal System.Process.Environment.OsString\n\
  \  System.Process.Internals\n\
  \semaphore-compat System.Semaphore\n\
  \stm\n\
  \  Control.Concurrent.STM Control.Concurrent.STM.TArray Control.Concurrent.STM.TBQueue\n\
  \  Control.Concurrent.STM.TChan Control.Concurrent.STM.TMVar\n\
  \  Control.Concurrent.STM.TQueue Control.Concurrent.STM.TSem Control.Concurrent.STM.TVar\n\
  \  Control.Monad.STM\n"

boot5 :: String
boot5 =
  "template-haskell\n\
  \  Language.Haskell.TH Language.Haskell.TH.CodeDo Language.Haskell.TH.LanguageExtensions\n\
  \  Language.Haskell.TH.Lib Language.Haskell.TH.Ppr Language.Haskell.TH.PprLib\n\
  \  Language.Haskell.TH.Quote Language.Haskell.TH.Syntax\n\
  \template-haskell-lift Language.Haskell.TH.Lift\n\
  \template-haskell-quasiquoter Language.Haskell.TH.QuasiQuoter\n\
  \text\n\
  \  Data.Text Data.Text.Array Data.Text.Encoding Data.Text.Encoding.Error\n\
  \  Data.Text.Foreign Data.Text.IO Data.Text.IO.Utf8 Data.Text.Internal\n\
  \  Data.Text.Internal.ArrayUtils Data.Text.Internal.Builder\n\
  \  Data.Text.Internal.Builder.Functions Data.Text.Internal.Builder.Int.Digits\n\
  \  Data.Text.Internal.Builder.RealFloat.Functions Data.Text.Internal.ByteStringCompat\n\
  \  Data.Text.Internal.Encoding Data.Text.Internal.Encoding.Fusion\n\
  \  Data.Text.Internal.Encoding.Fusion.Common Data.Text.Internal.Encoding.Utf16\n\
  \  Data.Text.Internal.Encoding.Utf32 Data.Text.Internal.Encoding.Utf8\n\
  \  Data.Text.Internal.Fusion Data.Text.Internal.Fusion.CaseMapping\n\
  \  Data.Text.Internal.Fusion.Common Data.Text.Internal.Fusion.Size\n\
  \  Data.Text.Internal.Fusion.Types Data.Text.Internal.IO Data.Text.Internal.Lazy\n\
  \  Data.Text.Internal.Lazy.Encoding.Fusion Data.Text.Internal.Lazy.Fusion\n\
  \  Data.Text.Internal.Lazy.Search Data.Text.Internal.PrimCompat\n\
  \  Data.Text.Internal.Private Data.Text.Internal.Read Data.Text.Internal.Search\n\
  \  Data.Text.Internal.StrictBuilder Data.Text.Internal.Unsafe\n\
  \  Data.Text.Internal.Unsafe.Char Data.Text.Internal.Validate\n\
  \  Data.Text.Internal.Validate.Native Data.Text.Lazy Data.Text.Lazy.Builder\n\
  \  Data.Text.Lazy.Builder.Int Data.Text.Lazy.Builder.RealFloat Data.Text.Lazy.Encoding\n\
  \  Data.Text.Lazy.IO Data.Text.Lazy.Internal Data.Text.Lazy.Read Data.Text.Read\n\
  \  Data.Text.Unsafe\n\
  \time\n\
  \  Data.Time Data.Time.Calendar Data.Time.Calendar.Easter Data.Time.Calendar.Julian\n\
  \  Data.Time.Calendar.Month Data.Time.Calendar.MonthDay Data.Time.Calendar.OrdinalDate\n\
  \  Data.Time.Calendar.Quarter Data.Time.Calendar.WeekDate Data.Time.Clock\n\
  \  Data.Time.Clock.POSIX Data.Time.Clock.System Data.Time.Clock.TAI Data.Time.Format\n\
  \  Data.Time.Format.ISO8601 Data.Time.LocalTime\n"

boot6 :: String
boot6 =
  "transformers\n\
  \  Control.Applicative.Backwards Control.Applicative.Lift Control.Monad.Signatures\n\
  \  Control.Monad.Trans.Accum Control.Monad.Trans.Class Control.Monad.Trans.Cont\n\
  \  Control.Monad.Trans.Except Control.Monad.Trans.Identity Control.Monad.Trans.Maybe\n\
  \  Control.Monad.Trans.RWS Control.Monad.Trans.RWS.CPS Control.Monad.Trans.RWS.Lazy\n\
  \  Control.Monad.Trans.RWS.Strict Control.Monad.Trans.Reader Control.Monad.Trans.Select\n\
  \  Control.Monad.Trans.State Control.Monad.Trans.State.Lazy\n\
  \  Control.Monad.Trans.State.Strict Control.Monad.Trans.Writer\n\
  \  Control.Monad.Trans.Writer.CPS Control.Monad.Trans.Writer.Lazy\n\
  \  Control.Monad.Trans.Writer.Strict Data.Functor.Constant Data.Functor.Reverse\n\
  \xhtml\n\
  \  Text.XHtml Text.XHtml.Debug Text.XHtml.Frameset Text.XHtml.Strict Text.XHtml.Table\n\
  \  Text.XHtml.Transitional\n"
