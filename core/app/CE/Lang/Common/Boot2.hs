-- | The GHC global package table, part 2 of 3 (plan v2.32 step 1),
-- transcribed from cli/src/graph/ladder/hs_boot.rs at a378e78c; from this
-- commit on the core is the authority.
module CE.Lang.Common.Boot2 where

boot :: String
boot =
  "Win32\n\
  \  Graphics.Win32 Graphics.Win32.Control Graphics.Win32.Dialogue Graphics.Win32.GDI\n\
  \  Graphics.Win32.GDI.AlphaBlend Graphics.Win32.GDI.Bitmap Graphics.Win32.GDI.Brush\n\
  \  Graphics.Win32.GDI.Clip Graphics.Win32.GDI.Font Graphics.Win32.GDI.Graphics2D\n\
  \  Graphics.Win32.GDI.HDC Graphics.Win32.GDI.Palette Graphics.Win32.GDI.Path\n\
  \  Graphics.Win32.GDI.Pen Graphics.Win32.GDI.Region Graphics.Win32.GDI.Types\n\
  \  Graphics.Win32.Icon Graphics.Win32.Key Graphics.Win32.LayeredWindow\n\
  \  Graphics.Win32.Menu Graphics.Win32.Message Graphics.Win32.Misc Graphics.Win32.Resource\n\
  \  Graphics.Win32.Window Graphics.Win32.Window.AnimateWindow\n\
  \  Graphics.Win32.Window.ForegroundWindow Graphics.Win32.Window.HotKey\n\
  \  Graphics.Win32.Window.IMM Graphics.Win32.Window.PostMessage Media.Win32 System.Win32\n\
  \  System.Win32.Automation System.Win32.Automation.Input\n\
  \  System.Win32.Automation.Input.Key System.Win32.Automation.Input.Mouse\n\
  \  System.Win32.Console System.Win32.Console.CtrlHandler System.Win32.Console.HWND\n\
  \  System.Win32.Console.Title System.Win32.DLL System.Win32.DebugApi\n\
  \  System.Win32.Encoding System.Win32.Event System.Win32.Exception.Unsupported\n\
  \  System.Win32.File System.Win32.FileMapping System.Win32.HardLink System.Win32.Info\n\
  \  System.Win32.Info.Computer System.Win32.Info.Version System.Win32.Mem\n\
  \  System.Win32.MinTTY System.Win32.NLS System.Win32.NamedPipes System.Win32.Path\n\
  \  System.Win32.Process System.Win32.Registry System.Win32.Security\n\
  \  System.Win32.Semaphore System.Win32.Shell System.Win32.SimpleMAPI System.Win32.String\n\
  \  System.Win32.SymbolicLink System.Win32.Thread System.Win32.Time System.Win32.Types\n\
  \  System.Win32.Utils System.Win32.WindowsString.Console System.Win32.WindowsString.DLL\n\
  \  System.Win32.WindowsString.DebugApi System.Win32.WindowsString.File\n\
  \  System.Win32.WindowsString.FileMapping System.Win32.WindowsString.HardLink\n\
  \  System.Win32.WindowsString.Info System.Win32.WindowsString.Path\n\
  \  System.Win32.WindowsString.Shell System.Win32.WindowsString.String\n\
  \  System.Win32.WindowsString.SymbolicLink System.Win32.WindowsString.Time\n\
  \  System.Win32.WindowsString.Types System.Win32.WindowsString.Utils System.Win32.Word\n\
  \array\n\
  \  Data.Array Data.Array.Base Data.Array.IArray Data.Array.IO Data.Array.IO.Internals\n\
  \  Data.Array.IO.Safe Data.Array.MArray Data.Array.MArray.Safe Data.Array.ST\n\
  \  Data.Array.ST.Safe Data.Array.Storable Data.Array.Storable.Internals\n\
  \  Data.Array.Storable.Safe Data.Array.Unboxed Data.Array.Unsafe\n"

boot2 :: String
boot2 =
  "base\n\
  \  Control.Applicative Control.Arrow Control.Category Control.Concurrent\n\
  \  Control.Concurrent.Chan Control.Concurrent.MVar Control.Concurrent.QSem\n\
  \  Control.Concurrent.QSemN Control.Exception Control.Exception.Annotation\n\
  \  Control.Exception.Backtrace Control.Exception.Base Control.Exception.Context\n\
  \  Control.Monad Control.Monad.Fail Control.Monad.Fix Control.Monad.IO.Class\n\
  \  Control.Monad.Instances Control.Monad.ST Control.Monad.ST.Lazy\n\
  \  Control.Monad.ST.Lazy.Safe Control.Monad.ST.Lazy.Unsafe Control.Monad.ST.Safe\n\
  \  Control.Monad.ST.Strict Control.Monad.ST.Unsafe Control.Monad.Zip Data.Array.Byte\n\
  \  Data.Bifoldable Data.Bifoldable1 Data.Bifunctor Data.Bitraversable Data.Bits Data.Bool\n\
  \  Data.Bounded Data.Char Data.Coerce Data.Complex Data.Data Data.Dynamic Data.Either\n\
  \  Data.Enum Data.Eq Data.Fixed Data.Foldable Data.Foldable1 Data.Function Data.Functor\n\
  \  Data.Functor.Classes Data.Functor.Compose Data.Functor.Const\n\
  \  Data.Functor.Contravariant Data.Functor.Identity Data.Functor.Product Data.Functor.Sum\n\
  \  Data.IORef Data.Int Data.Ix Data.Kind Data.List Data.List.NonEmpty Data.Maybe\n\
  \  Data.Monoid Data.Ord Data.Proxy Data.Ratio Data.STRef Data.STRef.Lazy\n\
  \  Data.STRef.Strict Data.Semigroup Data.String Data.Traversable Data.Tuple\n\
  \  Data.Type.Bool Data.Type.Coercion Data.Type.Equality Data.Type.Ord Data.Typeable\n\
  \  Data.Unique Data.Version Data.Void Data.Word Debug.Trace Foreign Foreign.C\n\
  \  Foreign.C.ConstPtr Foreign.C.Error Foreign.C.String Foreign.C.Types Foreign.Concurrent\n\
  \  Foreign.ForeignPtr Foreign.ForeignPtr.Safe Foreign.ForeignPtr.Unsafe Foreign.Marshal\n\
  \  Foreign.Marshal.Alloc Foreign.Marshal.Array Foreign.Marshal.Error Foreign.Marshal.Pool\n\
  \  Foreign.Marshal.Safe Foreign.Marshal.Unsafe Foreign.Marshal.Utils Foreign.Ptr\n\
  \  Foreign.Safe Foreign.StablePtr Foreign.Storable GHC.Arr GHC.ArrayArray GHC.Base\n\
  \  GHC.Bits GHC.ByteOrder GHC.Char GHC.Clock GHC.Conc GHC.Conc.IO GHC.Conc.POSIX\n\
  \  GHC.Conc.POSIX.Const GHC.Conc.Signal GHC.Conc.Sync GHC.Conc.WinIO GHC.Conc.Windows\n\
  \  GHC.ConsoleHandler GHC.Constants GHC.Desugar GHC.Encoding.UTF8 GHC.Enum\n\
  \  GHC.Environment GHC.Err GHC.Event.TimeOut GHC.Event.Windows GHC.Event.Windows.Clock\n\
  \  GHC.Event.Windows.ConsoleEvent GHC.Event.Windows.FFI\n\
  \  GHC.Event.Windows.ManagedThreadPool GHC.Event.Windows.Thread GHC.Exception\n\
  \  GHC.Exception.Type GHC.ExecutionStack GHC.Exts GHC.Fingerprint GHC.Fingerprint.Type\n\
  \  GHC.Float GHC.Float.ConversionUtils GHC.Float.RealFracMethods GHC.Foreign\n\
  \  GHC.ForeignPtr GHC.GHCi GHC.GHCi.Helpers GHC.Generics GHC.IO GHC.IO.Buffer\n\
  \  GHC.IO.BufferedIO GHC.IO.Device GHC.IO.Encoding GHC.IO.Encoding.CodePage\n\
  \  GHC.IO.Encoding.CodePage.API GHC.IO.Encoding.CodePage.Table GHC.IO.Encoding.Failure\n\
  \  GHC.IO.Encoding.Iconv GHC.IO.Encoding.Latin1 GHC.IO.Encoding.Types\n\
  \  GHC.IO.Encoding.UTF16 GHC.IO.Encoding.UTF32 GHC.IO.Encoding.UTF8 GHC.IO.Exception\n\
  \  GHC.IO.FD GHC.IO.Handle GHC.IO.Handle.FD GHC.IO.Handle.Internals GHC.IO.Handle.Lock\n\
  \  GHC.IO.Handle.Text GHC.IO.Handle.Types GHC.IO.Handle.Windows GHC.IO.IOMode\n\
  \  GHC.IO.StdHandles GHC.IO.SubSystem GHC.IO.Unsafe GHC.IO.Windows.Encoding\n\
  \  GHC.IO.Windows.Handle GHC.IO.Windows.Paths GHC.IOArray GHC.IORef GHC.InfoProv GHC.Int\n\
  \  GHC.Integer GHC.Integer.Logarithms GHC.IsList GHC.Ix GHC.List GHC.MVar GHC.Maybe\n"

boot3 :: String
boot3 =
  "  GHC.Natural GHC.Num GHC.Num.BigNat GHC.Num.Integer GHC.Num.Natural GHC.OldList\n\
  \  GHC.OverloadedLabels GHC.Profiling GHC.Ptr GHC.RTS.Flags GHC.Read GHC.Real GHC.Records\n\
  \  GHC.ResponseFile GHC.ST GHC.STRef GHC.Show GHC.Stable GHC.StableName GHC.Stack\n\
  \  GHC.Stack.CCS GHC.Stack.CloneStack GHC.Stack.Types GHC.StaticPtr GHC.Stats\n\
  \  GHC.Storable GHC.TopHandler GHC.TypeError GHC.TypeLits GHC.TypeNats GHC.Unicode\n\
  \  GHC.Weak GHC.Weak.Finalize GHC.Windows GHC.Word Numeric Numeric.Natural Prelude\n\
  \  System.CPUTime System.Console.GetOpt System.Environment System.Environment.Blank\n\
  \  System.Exit System.IO System.IO.Error System.IO.Unsafe System.Info System.Mem\n\
  \  System.Mem.StableName System.Mem.Weak System.Posix.Internals System.Posix.Types\n\
  \  System.Timeout Text.ParserCombinators.ReadP Text.ParserCombinators.ReadPrec\n\
  \  Text.Printf Text.Read Text.Read.Lex Text.Show Text.Show.Functions Type.Reflection\n\
  \  Type.Reflection.Unsafe Unsafe.Coerce\n\
  \binary\n\
  \  Data.Binary Data.Binary.Builder Data.Binary.Get Data.Binary.Get.Internal\n\
  \  Data.Binary.Put\n\
  \bytestring\n\
  \  Data.ByteString Data.ByteString.Builder Data.ByteString.Builder.Extra\n\
  \  Data.ByteString.Builder.Internal Data.ByteString.Builder.Prim\n\
  \  Data.ByteString.Builder.Prim.Internal Data.ByteString.Builder.RealFloat\n\
  \  Data.ByteString.Char8 Data.ByteString.Internal Data.ByteString.Lazy\n\
  \  Data.ByteString.Lazy.Char8 Data.ByteString.Lazy.Internal Data.ByteString.Short\n\
  \  Data.ByteString.Short.Internal Data.ByteString.Unsafe\n\
  \containers\n\
  \  Data.Containers.ListUtils Data.Graph Data.IntMap Data.IntMap.Internal\n\
  \  Data.IntMap.Internal.Debug Data.IntMap.Lazy Data.IntMap.Merge.Lazy\n\
  \  Data.IntMap.Merge.Strict Data.IntMap.Strict Data.IntMap.Strict.Internal Data.IntSet\n\
  \  Data.IntSet.Internal Data.IntSet.Internal.IntTreeCommons Data.Map Data.Map.Internal\n\
  \  Data.Map.Internal.Debug Data.Map.Lazy Data.Map.Merge.Lazy Data.Map.Merge.Strict\n\
  \  Data.Map.Strict Data.Map.Strict.Internal Data.Sequence Data.Sequence.Internal\n\
  \  Data.Sequence.Internal.Sorting Data.Set Data.Set.Internal Data.Tree\n\
  \deepseq Control.DeepSeq\n\
  \directory\n\
  \  System.Directory System.Directory.Internal System.Directory.Internal.Prelude\n\
  \  System.Directory.OsPath\n\
  \exceptions Control.Monad.Catch Control.Monad.Catch.Pure\n\
  \file-io\n\
  \  System.File.OsPath System.File.OsPath.Internal System.File.PlatformPath\n\
  \  System.File.PlatformPath.Internal\n"

boot4 :: String
boot4 =
  "filepath\n\
  \  System.FilePath System.FilePath.Posix System.FilePath.Windows System.OsPath\n\
  \  System.OsPath.Encoding System.OsPath.Internal System.OsPath.Posix\n\
  \  System.OsPath.Posix.Internal System.OsPath.Types System.OsPath.Windows\n\
  \  System.OsPath.Windows.Internal\n\
  \ghc-bignum\n\
  \  GHC.Num.Backend GHC.Num.Backend.Native GHC.Num.Backend.Selected GHC.Num.BigNat\n\
  \  GHC.Num.Integer GHC.Num.Natural GHC.Num.Primitives GHC.Num.WordArray\n\
  \ghc-boot\n\
  \  GHC.BaseDir GHC.Data.ShortText GHC.Data.SizedSeq GHC.ForeignSrcLang\n\
  \  GHC.ForeignSrcLang.Type GHC.HandleEncoding GHC.LanguageExtensions\n\
  \  GHC.LanguageExtensions.Type GHC.Lexeme GHC.Platform.ArchOS GHC.Platform.Host\n\
  \  GHC.Serialized GHC.Settings.Utils GHC.UniqueSubdir GHC.Unit.Database\n\
  \  GHC.Utils.Encoding GHC.Utils.Encoding.UTF8 GHC.Version\n\
  \ghc-boot-th\n\
  \  GHC.Boot.TH.Lib GHC.Boot.TH.Lib.Map GHC.Boot.TH.Lift GHC.Boot.TH.Ppr\n\
  \  GHC.Boot.TH.PprLib GHC.Boot.TH.Quote GHC.Boot.TH.Syntax GHC.ForeignSrcLang.Type\n\
  \  GHC.LanguageExtensions.Type GHC.Lexeme\n\
  \ghc-compact GHC.Compact GHC.Compact.Serialized\n\
  \ghc-experimental\n\
  \  Data.Sum.Experimental Data.Tuple.Experimental GHC.PrimOps GHC.Profiling.Eras\n\
  \  GHC.RTS.Flags.Experimental GHC.Stack.Annotation.Experimental GHC.Stats.Experimental\n\
  \  GHC.TypeLits.Experimental GHC.TypeNats.Experimental Prelude.Experimental\n\
  \  System.Mem.Experimental\n\
  \ghc-heap\n\
  \  GHC.Exts.Heap GHC.Exts.Heap.ClosureTypes GHC.Exts.Heap.Closures\n\
  \  GHC.Exts.Heap.Constants GHC.Exts.Heap.FFIClosures\n\
  \  GHC.Exts.Heap.FFIClosures_ProfilingDisabled GHC.Exts.Heap.FFIClosures_ProfilingEnabled\n\
  \  GHC.Exts.Heap.InfoTable GHC.Exts.Heap.InfoTable.Types GHC.Exts.Heap.InfoTableProf\n\
  \  GHC.Exts.Heap.ProfInfo.PeekProfInfo\n\
  \  GHC.Exts.Heap.ProfInfo.PeekProfInfo_ProfilingDisabled\n\
  \  GHC.Exts.Heap.ProfInfo.PeekProfInfo_ProfilingEnabled GHC.Exts.Heap.ProfInfo.Types\n\
  \  GHC.Exts.Heap.Utils GHC.Exts.Stack GHC.Exts.Stack.Constants GHC.Exts.Stack.Decode\n"
