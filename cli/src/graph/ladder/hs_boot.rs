//! GHC 9.14.1 global-package-db exposed modules — machine-generated
//! (2026-08-14) by scratchpad gen_hs_boot.py via `ghc-pkg list
//! --global` + `ghc-pkg field <pkg> exposed,exposed-modules`, never
//! hand-typed (the Go `go list std` / Python stdlib_module_names
//! precedent). Selection is the db's own facts, zero curation: every
//! global package except ghc (hidden); rts (no exposed modules); system-cxx-std-lib (no exposed modules) —
//! the hidden bit is the same visibility fact cabal honours.
//! Re-exports keep their EXPOSED name (`Name from pkg:Orig` → Name).
//! A missing name degrades to Unresolved (precision-safe), visible
//! in the ledger, and is repaid by regenerating this table. Sheer
//! LENGTH is the data, not style — the E01 file axis reads >300
//! here as a table fact with this header as its inline why.

/// (package name, its exposed modules, space-separated) — the
/// external rung's evidence base: 43 packages, 1371 modules.
pub const BOOT: &[(&str, &str)] = &[
    (
        "Cabal",
        "\
        Distribution.Backpack Distribution.Backpack.ComponentsGraph \
        Distribution.Backpack.Configure Distribution.Backpack.ConfiguredComponent \
        Distribution.Backpack.DescribeUnitId Distribution.Backpack.FullUnitId \
        Distribution.Backpack.LinkedComponent Distribution.Backpack.ModSubst \
        Distribution.Backpack.ModuleShape Distribution.Backpack.PreModuleShape \
        Distribution.CabalSpecVersion Distribution.Compat.Binary \
        Distribution.Compat.CharParsing Distribution.Compat.CreatePipe \
        Distribution.Compat.DList Distribution.Compat.Directory Distribution.Compat.Environment \
        Distribution.Compat.Exception Distribution.Compat.FilePath Distribution.Compat.Graph \
        Distribution.Compat.Internal.TempFile Distribution.Compat.Lens \
        Distribution.Compat.MonadFail Distribution.Compat.Newtype \
        Distribution.Compat.NonEmptySet Distribution.Compat.Parsing Distribution.Compat.Prelude \
        Distribution.Compat.Prelude.Internal Distribution.Compat.Process \
        Distribution.Compat.ResponseFile Distribution.Compat.Semigroup \
        Distribution.Compat.Stack Distribution.Compat.Time Distribution.Compiler \
        Distribution.FieldGrammar Distribution.FieldGrammar.Class \
        Distribution.FieldGrammar.FieldDescrs Distribution.FieldGrammar.Newtypes \
        Distribution.FieldGrammar.Parsec Distribution.FieldGrammar.Pretty Distribution.Fields \
        Distribution.Fields.ConfVar Distribution.Fields.Field Distribution.Fields.Lexer \
        Distribution.Fields.LexerMonad Distribution.Fields.ParseResult \
        Distribution.Fields.Parser Distribution.Fields.Pretty Distribution.InstalledPackageInfo \
        Distribution.License Distribution.Make Distribution.ModuleName Distribution.Package \
        Distribution.PackageDescription Distribution.PackageDescription.Check \
        Distribution.PackageDescription.Configuration \
        Distribution.PackageDescription.FieldGrammar Distribution.PackageDescription.Parsec \
        Distribution.PackageDescription.PrettyPrint Distribution.PackageDescription.Quirks \
        Distribution.PackageDescription.Utils Distribution.Parsec Distribution.Parsec.Error \
        Distribution.Parsec.FieldLineStream Distribution.Parsec.Position \
        Distribution.Parsec.Warning Distribution.Pretty Distribution.ReadE Distribution.SPDX \
        Distribution.SPDX.License Distribution.SPDX.LicenseExceptionId \
        Distribution.SPDX.LicenseExpression Distribution.SPDX.LicenseId \
        Distribution.SPDX.LicenseListVersion Distribution.SPDX.LicenseReference \
        Distribution.Simple Distribution.Simple.Bench Distribution.Simple.Build \
        Distribution.Simple.Build.Inputs Distribution.Simple.Build.Macros \
        Distribution.Simple.Build.PackageInfoModule Distribution.Simple.Build.PathsModule \
        Distribution.Simple.BuildPaths Distribution.Simple.BuildTarget \
        Distribution.Simple.BuildToolDepends Distribution.Simple.BuildWay \
        Distribution.Simple.CCompiler Distribution.Simple.Command Distribution.Simple.Compiler \
        Distribution.Simple.Configure Distribution.Simple.Errors \
        Distribution.Simple.FileMonitor.Types Distribution.Simple.Flag Distribution.Simple.GHC \
        Distribution.Simple.GHCJS Distribution.Simple.Glob Distribution.Simple.Glob.Internal \
        Distribution.Simple.Haddock Distribution.Simple.Hpc Distribution.Simple.Install \
        Distribution.Simple.InstallDirs Distribution.Simple.InstallDirs.Internal \
        Distribution.Simple.LocalBuildInfo Distribution.Simple.PackageDescription \
        Distribution.Simple.PackageIndex Distribution.Simple.PreProcess \
        Distribution.Simple.PreProcess.Types Distribution.Simple.PreProcess.Unlit \
        Distribution.Simple.Program Distribution.Simple.Program.Ar \
        Distribution.Simple.Program.Builtin Distribution.Simple.Program.Db \
        Distribution.Simple.Program.Find Distribution.Simple.Program.GHC \
        Distribution.Simple.Program.HcPkg Distribution.Simple.Program.Hpc \
        Distribution.Simple.Program.Internal Distribution.Simple.Program.Ld \
        Distribution.Simple.Program.ResponseFile Distribution.Simple.Program.Run \
        Distribution.Simple.Program.Script Distribution.Simple.Program.Strip \
        Distribution.Simple.Program.Types Distribution.Simple.Register \
        Distribution.Simple.Setup Distribution.Simple.SetupHooks.Errors \
        Distribution.Simple.SetupHooks.Internal Distribution.Simple.SetupHooks.Rule \
        Distribution.Simple.ShowBuildInfo Distribution.Simple.SrcDist Distribution.Simple.Test \
        Distribution.Simple.Test.ExeV10 Distribution.Simple.Test.LibV09 \
        Distribution.Simple.Test.Log Distribution.Simple.UHC Distribution.Simple.UserHooks \
        Distribution.Simple.Utils Distribution.System Distribution.TestSuite Distribution.Text \
        Distribution.Types.AbiDependency Distribution.Types.AbiHash \
        Distribution.Types.AnnotatedId Distribution.Types.Benchmark \
        Distribution.Types.Benchmark.Lens Distribution.Types.BenchmarkInterface \
        Distribution.Types.BenchmarkType Distribution.Types.BuildInfo \
        Distribution.Types.BuildInfo.Lens Distribution.Types.BuildType \
        Distribution.Types.Component Distribution.Types.ComponentId \
        Distribution.Types.ComponentInclude Distribution.Types.ComponentLocalBuildInfo \
        Distribution.Types.ComponentName Distribution.Types.ComponentRequestedSpec \
        Distribution.Types.CondTree Distribution.Types.Condition Distribution.Types.ConfVar \
        Distribution.Types.Dependency Distribution.Types.DependencyMap \
        Distribution.Types.DependencySatisfaction Distribution.Types.DumpBuildInfo \
        Distribution.Types.ExeDependency Distribution.Types.Executable \
        Distribution.Types.Executable.Lens Distribution.Types.ExecutableScope \
        Distribution.Types.ExposedModule Distribution.Types.Flag Distribution.Types.ForeignLib \
        Distribution.Types.ForeignLib.Lens Distribution.Types.ForeignLibOption \
        Distribution.Types.ForeignLibType Distribution.Types.GenericPackageDescription \
        Distribution.Types.GenericPackageDescription.Lens Distribution.Types.GivenComponent \
        Distribution.Types.HookedBuildInfo Distribution.Types.IncludeRenaming \
        Distribution.Types.InstalledPackageInfo \
        Distribution.Types.InstalledPackageInfo.FieldGrammar \
        Distribution.Types.InstalledPackageInfo.Lens Distribution.Types.LegacyExeDependency \
        Distribution.Types.Lens Distribution.Types.Library Distribution.Types.Library.Lens \
        Distribution.Types.LibraryName Distribution.Types.LibraryVisibility \
        Distribution.Types.LocalBuildConfig Distribution.Types.LocalBuildInfo \
        Distribution.Types.MissingDependency Distribution.Types.MissingDependencyReason \
        Distribution.Types.Mixin Distribution.Types.Module Distribution.Types.ModuleReexport \
        Distribution.Types.ModuleRenaming Distribution.Types.MungedPackageId \
        Distribution.Types.MungedPackageName Distribution.Types.PackageDescription \
        Distribution.Types.PackageDescription.Lens Distribution.Types.PackageId \
        Distribution.Types.PackageId.Lens Distribution.Types.PackageName \
        Distribution.Types.PackageName.Magic Distribution.Types.PackageVersionConstraint \
        Distribution.Types.ParStrat Distribution.Types.PkgconfigDependency \
        Distribution.Types.PkgconfigName Distribution.Types.PkgconfigVersion \
        Distribution.Types.PkgconfigVersionRange Distribution.Types.SetupBuildInfo \
        Distribution.Types.SetupBuildInfo.Lens Distribution.Types.SourceRepo \
        Distribution.Types.SourceRepo.Lens Distribution.Types.TargetInfo \
        Distribution.Types.TestSuite Distribution.Types.TestSuite.Lens \
        Distribution.Types.TestSuiteInterface Distribution.Types.TestType \
        Distribution.Types.UnitId Distribution.Types.UnqualComponentName \
        Distribution.Types.Version Distribution.Types.VersionInterval \
        Distribution.Types.VersionInterval.Legacy Distribution.Types.VersionRange \
        Distribution.Types.VersionRange.Internal Distribution.Utils.Base62 \
        Distribution.Utils.Generic Distribution.Utils.IOData Distribution.Utils.Json \
        Distribution.Utils.LogProgress Distribution.Utils.MD5 Distribution.Utils.MapAccum \
        Distribution.Utils.NubList Distribution.Utils.Path Distribution.Utils.Progress \
        Distribution.Utils.ShortText Distribution.Utils.String Distribution.Utils.Structured \
        Distribution.Verbosity Distribution.Verbosity.Internal Distribution.Version \
        Language.Haskell.Extension",
    ),
    (
        "Cabal-syntax",
        "\
        Distribution.Backpack Distribution.CabalSpecVersion Distribution.Compat.Binary \
        Distribution.Compat.CharParsing Distribution.Compat.DList Distribution.Compat.Exception \
        Distribution.Compat.Graph Distribution.Compat.Lens Distribution.Compat.MonadFail \
        Distribution.Compat.Newtype Distribution.Compat.NonEmptySet Distribution.Compat.Parsing \
        Distribution.Compat.Prelude Distribution.Compat.Semigroup Distribution.Compiler \
        Distribution.FieldGrammar Distribution.FieldGrammar.Class \
        Distribution.FieldGrammar.FieldDescrs Distribution.FieldGrammar.Newtypes \
        Distribution.FieldGrammar.Parsec Distribution.FieldGrammar.Pretty Distribution.Fields \
        Distribution.Fields.ConfVar Distribution.Fields.Field Distribution.Fields.Lexer \
        Distribution.Fields.LexerMonad Distribution.Fields.ParseResult \
        Distribution.Fields.Parser Distribution.Fields.Pretty Distribution.InstalledPackageInfo \
        Distribution.License Distribution.ModuleName Distribution.Package \
        Distribution.PackageDescription Distribution.PackageDescription.Configuration \
        Distribution.PackageDescription.FieldGrammar Distribution.PackageDescription.Parsec \
        Distribution.PackageDescription.PrettyPrint Distribution.PackageDescription.Quirks \
        Distribution.PackageDescription.Utils Distribution.Parsec Distribution.Parsec.Error \
        Distribution.Parsec.FieldLineStream Distribution.Parsec.Position \
        Distribution.Parsec.Warning Distribution.Pretty Distribution.SPDX \
        Distribution.SPDX.License Distribution.SPDX.LicenseExceptionId \
        Distribution.SPDX.LicenseExpression Distribution.SPDX.LicenseId \
        Distribution.SPDX.LicenseListVersion Distribution.SPDX.LicenseReference \
        Distribution.System Distribution.Text Distribution.Types.AbiDependency \
        Distribution.Types.AbiHash Distribution.Types.Benchmark \
        Distribution.Types.Benchmark.Lens Distribution.Types.BenchmarkInterface \
        Distribution.Types.BenchmarkType Distribution.Types.BuildInfo \
        Distribution.Types.BuildInfo.Lens Distribution.Types.BuildType \
        Distribution.Types.Component Distribution.Types.ComponentId \
        Distribution.Types.ComponentName Distribution.Types.ComponentRequestedSpec \
        Distribution.Types.CondTree Distribution.Types.Condition Distribution.Types.ConfVar \
        Distribution.Types.Dependency Distribution.Types.DependencyMap \
        Distribution.Types.DependencySatisfaction Distribution.Types.ExeDependency \
        Distribution.Types.Executable Distribution.Types.Executable.Lens \
        Distribution.Types.ExecutableScope Distribution.Types.ExposedModule \
        Distribution.Types.Flag Distribution.Types.ForeignLib \
        Distribution.Types.ForeignLib.Lens Distribution.Types.ForeignLibOption \
        Distribution.Types.ForeignLibType Distribution.Types.GenericPackageDescription \
        Distribution.Types.GenericPackageDescription.Lens Distribution.Types.HookedBuildInfo \
        Distribution.Types.IncludeRenaming Distribution.Types.InstalledPackageInfo \
        Distribution.Types.InstalledPackageInfo.FieldGrammar \
        Distribution.Types.InstalledPackageInfo.Lens Distribution.Types.LegacyExeDependency \
        Distribution.Types.Lens Distribution.Types.Library Distribution.Types.Library.Lens \
        Distribution.Types.LibraryName Distribution.Types.LibraryVisibility \
        Distribution.Types.MissingDependency Distribution.Types.MissingDependencyReason \
        Distribution.Types.Mixin Distribution.Types.Module Distribution.Types.ModuleReexport \
        Distribution.Types.ModuleRenaming Distribution.Types.MungedPackageId \
        Distribution.Types.MungedPackageName Distribution.Types.PackageDescription \
        Distribution.Types.PackageDescription.Lens Distribution.Types.PackageId \
        Distribution.Types.PackageId.Lens Distribution.Types.PackageName \
        Distribution.Types.PackageVersionConstraint Distribution.Types.PkgconfigDependency \
        Distribution.Types.PkgconfigName Distribution.Types.PkgconfigVersion \
        Distribution.Types.PkgconfigVersionRange Distribution.Types.SetupBuildInfo \
        Distribution.Types.SetupBuildInfo.Lens Distribution.Types.SourceRepo \
        Distribution.Types.SourceRepo.Lens Distribution.Types.TestSuite \
        Distribution.Types.TestSuite.Lens Distribution.Types.TestSuiteInterface \
        Distribution.Types.TestType Distribution.Types.UnitId \
        Distribution.Types.UnqualComponentName Distribution.Types.Version \
        Distribution.Types.VersionInterval Distribution.Types.VersionInterval.Legacy \
        Distribution.Types.VersionRange Distribution.Types.VersionRange.Internal \
        Distribution.Utils.Base62 Distribution.Utils.Generic Distribution.Utils.MD5 \
        Distribution.Utils.Path Distribution.Utils.ShortText Distribution.Utils.String \
        Distribution.Utils.Structured Distribution.Version Language.Haskell.Extension",
    ),
    (
        "Win32",
        "\
        Graphics.Win32 Graphics.Win32.Control Graphics.Win32.Dialogue Graphics.Win32.GDI \
        Graphics.Win32.GDI.AlphaBlend Graphics.Win32.GDI.Bitmap Graphics.Win32.GDI.Brush \
        Graphics.Win32.GDI.Clip Graphics.Win32.GDI.Font Graphics.Win32.GDI.Graphics2D \
        Graphics.Win32.GDI.HDC Graphics.Win32.GDI.Palette Graphics.Win32.GDI.Path \
        Graphics.Win32.GDI.Pen Graphics.Win32.GDI.Region Graphics.Win32.GDI.Types \
        Graphics.Win32.Icon Graphics.Win32.Key Graphics.Win32.LayeredWindow Graphics.Win32.Menu \
        Graphics.Win32.Message Graphics.Win32.Misc Graphics.Win32.Resource \
        Graphics.Win32.Window Graphics.Win32.Window.AnimateWindow \
        Graphics.Win32.Window.ForegroundWindow Graphics.Win32.Window.HotKey \
        Graphics.Win32.Window.IMM Graphics.Win32.Window.PostMessage Media.Win32 System.Win32 \
        System.Win32.Automation System.Win32.Automation.Input System.Win32.Automation.Input.Key \
        System.Win32.Automation.Input.Mouse System.Win32.Console \
        System.Win32.Console.CtrlHandler System.Win32.Console.HWND System.Win32.Console.Title \
        System.Win32.DLL System.Win32.DebugApi System.Win32.Encoding System.Win32.Event \
        System.Win32.Exception.Unsupported System.Win32.File System.Win32.FileMapping \
        System.Win32.HardLink System.Win32.Info System.Win32.Info.Computer \
        System.Win32.Info.Version System.Win32.Mem System.Win32.MinTTY System.Win32.NLS \
        System.Win32.NamedPipes System.Win32.Path System.Win32.Process System.Win32.Registry \
        System.Win32.Security System.Win32.Semaphore System.Win32.Shell System.Win32.SimpleMAPI \
        System.Win32.String System.Win32.SymbolicLink System.Win32.Thread System.Win32.Time \
        System.Win32.Types System.Win32.Utils System.Win32.WindowsString.Console \
        System.Win32.WindowsString.DLL System.Win32.WindowsString.DebugApi \
        System.Win32.WindowsString.File System.Win32.WindowsString.FileMapping \
        System.Win32.WindowsString.HardLink System.Win32.WindowsString.Info \
        System.Win32.WindowsString.Path System.Win32.WindowsString.Shell \
        System.Win32.WindowsString.String System.Win32.WindowsString.SymbolicLink \
        System.Win32.WindowsString.Time System.Win32.WindowsString.Types \
        System.Win32.WindowsString.Utils System.Win32.Word",
    ),
    (
        "array",
        "\
        Data.Array Data.Array.Base Data.Array.IArray Data.Array.IO Data.Array.IO.Internals \
        Data.Array.IO.Safe Data.Array.MArray Data.Array.MArray.Safe Data.Array.ST \
        Data.Array.ST.Safe Data.Array.Storable Data.Array.Storable.Internals \
        Data.Array.Storable.Safe Data.Array.Unboxed Data.Array.Unsafe",
    ),
    (
        "base",
        "\
        Control.Applicative Control.Arrow Control.Category Control.Concurrent \
        Control.Concurrent.Chan Control.Concurrent.MVar Control.Concurrent.QSem \
        Control.Concurrent.QSemN Control.Exception Control.Exception.Annotation \
        Control.Exception.Backtrace Control.Exception.Base Control.Exception.Context \
        Control.Monad Control.Monad.Fail Control.Monad.Fix Control.Monad.IO.Class \
        Control.Monad.Instances Control.Monad.ST Control.Monad.ST.Lazy \
        Control.Monad.ST.Lazy.Safe Control.Monad.ST.Lazy.Unsafe Control.Monad.ST.Safe \
        Control.Monad.ST.Strict Control.Monad.ST.Unsafe Control.Monad.Zip Data.Array.Byte \
        Data.Bifoldable Data.Bifoldable1 Data.Bifunctor Data.Bitraversable Data.Bits Data.Bool \
        Data.Bounded Data.Char Data.Coerce Data.Complex Data.Data Data.Dynamic Data.Either \
        Data.Enum Data.Eq Data.Fixed Data.Foldable Data.Foldable1 Data.Function Data.Functor \
        Data.Functor.Classes Data.Functor.Compose Data.Functor.Const Data.Functor.Contravariant \
        Data.Functor.Identity Data.Functor.Product Data.Functor.Sum Data.IORef Data.Int Data.Ix \
        Data.Kind Data.List Data.List.NonEmpty Data.Maybe Data.Monoid Data.Ord Data.Proxy \
        Data.Ratio Data.STRef Data.STRef.Lazy Data.STRef.Strict Data.Semigroup Data.String \
        Data.Traversable Data.Tuple Data.Type.Bool Data.Type.Coercion Data.Type.Equality \
        Data.Type.Ord Data.Typeable Data.Unique Data.Version Data.Void Data.Word Debug.Trace \
        Foreign Foreign.C Foreign.C.ConstPtr Foreign.C.Error Foreign.C.String Foreign.C.Types \
        Foreign.Concurrent Foreign.ForeignPtr Foreign.ForeignPtr.Safe Foreign.ForeignPtr.Unsafe \
        Foreign.Marshal Foreign.Marshal.Alloc Foreign.Marshal.Array Foreign.Marshal.Error \
        Foreign.Marshal.Pool Foreign.Marshal.Safe Foreign.Marshal.Unsafe Foreign.Marshal.Utils \
        Foreign.Ptr Foreign.Safe Foreign.StablePtr Foreign.Storable GHC.Arr GHC.ArrayArray \
        GHC.Base GHC.Bits GHC.ByteOrder GHC.Char GHC.Clock GHC.Conc GHC.Conc.IO GHC.Conc.POSIX \
        GHC.Conc.POSIX.Const GHC.Conc.Signal GHC.Conc.Sync GHC.Conc.WinIO GHC.Conc.Windows \
        GHC.ConsoleHandler GHC.Constants GHC.Desugar GHC.Encoding.UTF8 GHC.Enum GHC.Environment \
        GHC.Err GHC.Event.TimeOut GHC.Event.Windows GHC.Event.Windows.Clock \
        GHC.Event.Windows.ConsoleEvent GHC.Event.Windows.FFI \
        GHC.Event.Windows.ManagedThreadPool GHC.Event.Windows.Thread GHC.Exception \
        GHC.Exception.Type GHC.ExecutionStack GHC.Exts GHC.Fingerprint GHC.Fingerprint.Type \
        GHC.Float GHC.Float.ConversionUtils GHC.Float.RealFracMethods GHC.Foreign \
        GHC.ForeignPtr GHC.GHCi GHC.GHCi.Helpers GHC.Generics GHC.IO GHC.IO.Buffer \
        GHC.IO.BufferedIO GHC.IO.Device GHC.IO.Encoding GHC.IO.Encoding.CodePage \
        GHC.IO.Encoding.CodePage.API GHC.IO.Encoding.CodePage.Table GHC.IO.Encoding.Failure \
        GHC.IO.Encoding.Iconv GHC.IO.Encoding.Latin1 GHC.IO.Encoding.Types \
        GHC.IO.Encoding.UTF16 GHC.IO.Encoding.UTF32 GHC.IO.Encoding.UTF8 GHC.IO.Exception \
        GHC.IO.FD GHC.IO.Handle GHC.IO.Handle.FD GHC.IO.Handle.Internals GHC.IO.Handle.Lock \
        GHC.IO.Handle.Text GHC.IO.Handle.Types GHC.IO.Handle.Windows GHC.IO.IOMode \
        GHC.IO.StdHandles GHC.IO.SubSystem GHC.IO.Unsafe GHC.IO.Windows.Encoding \
        GHC.IO.Windows.Handle GHC.IO.Windows.Paths GHC.IOArray GHC.IORef GHC.InfoProv GHC.Int \
        GHC.Integer GHC.Integer.Logarithms GHC.IsList GHC.Ix GHC.List GHC.MVar GHC.Maybe \
        GHC.Natural GHC.Num GHC.Num.BigNat GHC.Num.Integer GHC.Num.Natural GHC.OldList \
        GHC.OverloadedLabels GHC.Profiling GHC.Ptr GHC.RTS.Flags GHC.Read GHC.Real GHC.Records \
        GHC.ResponseFile GHC.ST GHC.STRef GHC.Show GHC.Stable GHC.StableName GHC.Stack \
        GHC.Stack.CCS GHC.Stack.CloneStack GHC.Stack.Types GHC.StaticPtr GHC.Stats GHC.Storable \
        GHC.TopHandler GHC.TypeError GHC.TypeLits GHC.TypeNats GHC.Unicode GHC.Weak \
        GHC.Weak.Finalize GHC.Windows GHC.Word Numeric Numeric.Natural Prelude System.CPUTime \
        System.Console.GetOpt System.Environment System.Environment.Blank System.Exit System.IO \
        System.IO.Error System.IO.Unsafe System.Info System.Mem System.Mem.StableName \
        System.Mem.Weak System.Posix.Internals System.Posix.Types System.Timeout \
        Text.ParserCombinators.ReadP Text.ParserCombinators.ReadPrec Text.Printf Text.Read \
        Text.Read.Lex Text.Show Text.Show.Functions Type.Reflection Type.Reflection.Unsafe \
        Unsafe.Coerce",
    ),
    (
        "binary",
        "\
        Data.Binary Data.Binary.Builder Data.Binary.Get Data.Binary.Get.Internal \
        Data.Binary.Put",
    ),
    (
        "bytestring",
        "\
        Data.ByteString Data.ByteString.Builder Data.ByteString.Builder.Extra \
        Data.ByteString.Builder.Internal Data.ByteString.Builder.Prim \
        Data.ByteString.Builder.Prim.Internal Data.ByteString.Builder.RealFloat \
        Data.ByteString.Char8 Data.ByteString.Internal Data.ByteString.Lazy \
        Data.ByteString.Lazy.Char8 Data.ByteString.Lazy.Internal Data.ByteString.Short \
        Data.ByteString.Short.Internal Data.ByteString.Unsafe",
    ),
    (
        "containers",
        "\
        Data.Containers.ListUtils Data.Graph Data.IntMap Data.IntMap.Internal \
        Data.IntMap.Internal.Debug Data.IntMap.Lazy Data.IntMap.Merge.Lazy \
        Data.IntMap.Merge.Strict Data.IntMap.Strict Data.IntMap.Strict.Internal Data.IntSet \
        Data.IntSet.Internal Data.IntSet.Internal.IntTreeCommons Data.Map Data.Map.Internal \
        Data.Map.Internal.Debug Data.Map.Lazy Data.Map.Merge.Lazy Data.Map.Merge.Strict \
        Data.Map.Strict Data.Map.Strict.Internal Data.Sequence Data.Sequence.Internal \
        Data.Sequence.Internal.Sorting Data.Set Data.Set.Internal Data.Tree",
    ),
    (
        "deepseq",
        "\
        Control.DeepSeq",
    ),
    (
        "directory",
        "\
        System.Directory System.Directory.Internal System.Directory.Internal.Prelude \
        System.Directory.OsPath",
    ),
    (
        "exceptions",
        "\
        Control.Monad.Catch Control.Monad.Catch.Pure",
    ),
    (
        "file-io",
        "\
        System.File.OsPath System.File.OsPath.Internal System.File.PlatformPath \
        System.File.PlatformPath.Internal",
    ),
    (
        "filepath",
        "\
        System.FilePath System.FilePath.Posix System.FilePath.Windows System.OsPath \
        System.OsPath.Encoding System.OsPath.Internal System.OsPath.Posix \
        System.OsPath.Posix.Internal System.OsPath.Types System.OsPath.Windows \
        System.OsPath.Windows.Internal",
    ),
    (
        "ghc-bignum",
        "\
        GHC.Num.Backend GHC.Num.Backend.Native GHC.Num.Backend.Selected GHC.Num.BigNat \
        GHC.Num.Integer GHC.Num.Natural GHC.Num.Primitives GHC.Num.WordArray",
    ),
    (
        "ghc-boot",
        "\
        GHC.BaseDir GHC.Data.ShortText GHC.Data.SizedSeq GHC.ForeignSrcLang \
        GHC.ForeignSrcLang.Type GHC.HandleEncoding GHC.LanguageExtensions \
        GHC.LanguageExtensions.Type GHC.Lexeme GHC.Platform.ArchOS GHC.Platform.Host \
        GHC.Serialized GHC.Settings.Utils GHC.UniqueSubdir GHC.Unit.Database GHC.Utils.Encoding \
        GHC.Utils.Encoding.UTF8 GHC.Version",
    ),
    (
        "ghc-boot-th",
        "\
        GHC.Boot.TH.Lib GHC.Boot.TH.Lib.Map GHC.Boot.TH.Lift GHC.Boot.TH.Ppr GHC.Boot.TH.PprLib \
        GHC.Boot.TH.Quote GHC.Boot.TH.Syntax GHC.ForeignSrcLang.Type \
        GHC.LanguageExtensions.Type GHC.Lexeme",
    ),
    (
        "ghc-compact",
        "\
        GHC.Compact GHC.Compact.Serialized",
    ),
    (
        "ghc-experimental",
        "\
        Data.Sum.Experimental Data.Tuple.Experimental GHC.PrimOps GHC.Profiling.Eras \
        GHC.RTS.Flags.Experimental GHC.Stack.Annotation.Experimental GHC.Stats.Experimental \
        GHC.TypeLits.Experimental GHC.TypeNats.Experimental Prelude.Experimental \
        System.Mem.Experimental",
    ),
    (
        "ghc-heap",
        "\
        GHC.Exts.Heap GHC.Exts.Heap.ClosureTypes GHC.Exts.Heap.Closures GHC.Exts.Heap.Constants \
        GHC.Exts.Heap.FFIClosures GHC.Exts.Heap.FFIClosures_ProfilingDisabled \
        GHC.Exts.Heap.FFIClosures_ProfilingEnabled GHC.Exts.Heap.InfoTable \
        GHC.Exts.Heap.InfoTable.Types GHC.Exts.Heap.InfoTableProf \
        GHC.Exts.Heap.ProfInfo.PeekProfInfo \
        GHC.Exts.Heap.ProfInfo.PeekProfInfo_ProfilingDisabled \
        GHC.Exts.Heap.ProfInfo.PeekProfInfo_ProfilingEnabled GHC.Exts.Heap.ProfInfo.Types \
        GHC.Exts.Heap.Utils GHC.Exts.Stack GHC.Exts.Stack.Constants GHC.Exts.Stack.Decode",
    ),
    (
        "ghc-internal",
        "\
        GHC.Internal.AllocationLimitHandler GHC.Internal.Arr GHC.Internal.ArrayArray \
        GHC.Internal.Base GHC.Internal.Bignum.Backend GHC.Internal.Bignum.Backend.Native \
        GHC.Internal.Bignum.Backend.Selected GHC.Internal.Bignum.BigNat \
        GHC.Internal.Bignum.Integer GHC.Internal.Bignum.Natural GHC.Internal.Bignum.Primitives \
        GHC.Internal.Bignum.WordArray GHC.Internal.Bits GHC.Internal.ByteOrder \
        GHC.Internal.CString GHC.Internal.Char GHC.Internal.Classes GHC.Internal.Clock \
        GHC.Internal.ClosureTypes GHC.Internal.Conc.Bound GHC.Internal.Conc.IO \
        GHC.Internal.Conc.POSIX GHC.Internal.Conc.POSIX.Const GHC.Internal.Conc.Signal \
        GHC.Internal.Conc.Sync GHC.Internal.Conc.Windows GHC.Internal.ConsoleHandler \
        GHC.Internal.Control.Arrow GHC.Internal.Control.Category \
        GHC.Internal.Control.Concurrent.MVar GHC.Internal.Control.Exception \
        GHC.Internal.Control.Exception.Base GHC.Internal.Control.Monad \
        GHC.Internal.Control.Monad.Fail GHC.Internal.Control.Monad.Fix \
        GHC.Internal.Control.Monad.IO.Class GHC.Internal.Control.Monad.ST \
        GHC.Internal.Control.Monad.ST.Imp GHC.Internal.Control.Monad.ST.Lazy \
        GHC.Internal.Control.Monad.ST.Lazy.Imp GHC.Internal.Control.Monad.Zip \
        GHC.Internal.Data.Bits GHC.Internal.Data.Bool GHC.Internal.Data.Coerce \
        GHC.Internal.Data.Data GHC.Internal.Data.Dynamic GHC.Internal.Data.Either \
        GHC.Internal.Data.Eq GHC.Internal.Data.Foldable GHC.Internal.Data.Function \
        GHC.Internal.Data.Functor GHC.Internal.Data.Functor.Const \
        GHC.Internal.Data.Functor.Identity GHC.Internal.Data.Functor.Utils \
        GHC.Internal.Data.IORef GHC.Internal.Data.Ix GHC.Internal.Data.List \
        GHC.Internal.Data.List.NonEmpty GHC.Internal.Data.Maybe GHC.Internal.Data.Monoid \
        GHC.Internal.Data.NonEmpty GHC.Internal.Data.OldList GHC.Internal.Data.Ord \
        GHC.Internal.Data.Proxy GHC.Internal.Data.STRef GHC.Internal.Data.STRef.Strict \
        GHC.Internal.Data.Semigroup.Internal GHC.Internal.Data.String \
        GHC.Internal.Data.Traversable GHC.Internal.Data.Tuple GHC.Internal.Data.Type.Bool \
        GHC.Internal.Data.Type.Coercion GHC.Internal.Data.Type.Equality \
        GHC.Internal.Data.Type.Ord GHC.Internal.Data.Typeable GHC.Internal.Data.Unique \
        GHC.Internal.Data.Version GHC.Internal.Data.Void GHC.Internal.Debug \
        GHC.Internal.Debug.Trace GHC.Internal.Desugar GHC.Internal.Encoding.UTF8 \
        GHC.Internal.Enum GHC.Internal.Environment GHC.Internal.Err GHC.Internal.Event.TimeOut \
        GHC.Internal.Event.Windows GHC.Internal.Event.Windows.Clock \
        GHC.Internal.Event.Windows.ConsoleEvent GHC.Internal.Event.Windows.FFI \
        GHC.Internal.Event.Windows.ManagedThreadPool GHC.Internal.Event.Windows.Thread \
        GHC.Internal.Exception GHC.Internal.Exception.Backtrace GHC.Internal.Exception.Context \
        GHC.Internal.Exception.Type GHC.Internal.ExecutionStack \
        GHC.Internal.ExecutionStack.Internal GHC.Internal.Exts GHC.Internal.Fingerprint \
        GHC.Internal.Fingerprint.Type GHC.Internal.Float GHC.Internal.Float.ConversionUtils \
        GHC.Internal.Float.RealFracMethods GHC.Internal.Foreign.C.ConstPtr \
        GHC.Internal.Foreign.C.Error GHC.Internal.Foreign.C.String \
        GHC.Internal.Foreign.C.String.Encoding GHC.Internal.Foreign.C.Types \
        GHC.Internal.Foreign.Concurrent GHC.Internal.Foreign.ForeignPtr \
        GHC.Internal.Foreign.ForeignPtr.Imp GHC.Internal.Foreign.ForeignPtr.Unsafe \
        GHC.Internal.Foreign.Marshal.Alloc GHC.Internal.Foreign.Marshal.Array \
        GHC.Internal.Foreign.Marshal.Error GHC.Internal.Foreign.Marshal.Pool \
        GHC.Internal.Foreign.Marshal.Safe GHC.Internal.Foreign.Marshal.Unsafe \
        GHC.Internal.Foreign.Marshal.Utils GHC.Internal.Foreign.Ptr \
        GHC.Internal.Foreign.StablePtr GHC.Internal.Foreign.Storable GHC.Internal.ForeignPtr \
        GHC.Internal.ForeignSrcLang GHC.Internal.Functor.ZipList GHC.Internal.GHCi \
        GHC.Internal.GHCi.Helpers GHC.Internal.Generics GHC.Internal.Heap.Closures \
        GHC.Internal.Heap.Constants GHC.Internal.Heap.InfoTable \
        GHC.Internal.Heap.InfoTable.Types GHC.Internal.Heap.InfoTableProf \
        GHC.Internal.Heap.ProfInfo.Types GHC.Internal.IO GHC.Internal.IO.Buffer \
        GHC.Internal.IO.BufferedIO GHC.Internal.IO.Device GHC.Internal.IO.Encoding \
        GHC.Internal.IO.Encoding.CodePage GHC.Internal.IO.Encoding.CodePage.API \
        GHC.Internal.IO.Encoding.CodePage.Table GHC.Internal.IO.Encoding.Failure \
        GHC.Internal.IO.Encoding.Iconv GHC.Internal.IO.Encoding.Latin1 \
        GHC.Internal.IO.Encoding.Types GHC.Internal.IO.Encoding.UTF16 \
        GHC.Internal.IO.Encoding.UTF32 GHC.Internal.IO.Encoding.UTF8 GHC.Internal.IO.Exception \
        GHC.Internal.IO.FD GHC.Internal.IO.Handle GHC.Internal.IO.Handle.FD \
        GHC.Internal.IO.Handle.Internals GHC.Internal.IO.Handle.Lock \
        GHC.Internal.IO.Handle.Text GHC.Internal.IO.Handle.Types GHC.Internal.IO.Handle.Windows \
        GHC.Internal.IO.IOMode GHC.Internal.IO.StdHandles GHC.Internal.IO.SubSystem \
        GHC.Internal.IO.Unsafe GHC.Internal.IO.Windows.Encoding GHC.Internal.IO.Windows.Handle \
        GHC.Internal.IO.Windows.Paths GHC.Internal.IOArray GHC.Internal.IORef \
        GHC.Internal.InfoProv GHC.Internal.InfoProv.Types GHC.Internal.Int GHC.Internal.Integer \
        GHC.Internal.Integer.Logarithms GHC.Internal.IsList GHC.Internal.Ix \
        GHC.Internal.LanguageExtensions GHC.Internal.Lexeme GHC.Internal.List GHC.Internal.MVar \
        GHC.Internal.Magic GHC.Internal.Magic.Dict GHC.Internal.Maybe GHC.Internal.Natural \
        GHC.Internal.Num GHC.Internal.Numeric GHC.Internal.Numeric.Natural \
        GHC.Internal.OverloadedLabels GHC.Internal.Pack GHC.Internal.Prim \
        GHC.Internal.Prim.Exception GHC.Internal.Prim.Ext GHC.Internal.Prim.Panic \
        GHC.Internal.Prim.PtrEq GHC.Internal.PrimopWrappers GHC.Internal.Profiling \
        GHC.Internal.Ptr GHC.Internal.RTS.Flags GHC.Internal.RTS.Flags.Test GHC.Internal.Read \
        GHC.Internal.Real GHC.Internal.Records GHC.Internal.ResponseFile GHC.Internal.ST \
        GHC.Internal.STRef GHC.Internal.Show GHC.Internal.Stable GHC.Internal.StableName \
        GHC.Internal.Stack GHC.Internal.Stack.Annotation GHC.Internal.Stack.CCS \
        GHC.Internal.Stack.CloneStack GHC.Internal.Stack.Constants \
        GHC.Internal.Stack.ConstantsProf GHC.Internal.Stack.Decode GHC.Internal.Stack.Types \
        GHC.Internal.StaticPtr GHC.Internal.Stats GHC.Internal.Storable \
        GHC.Internal.System.Environment GHC.Internal.System.Environment.Blank \
        GHC.Internal.System.Exit GHC.Internal.System.IO GHC.Internal.System.IO.Error \
        GHC.Internal.System.Mem GHC.Internal.System.Mem.StableName \
        GHC.Internal.System.Posix.Internals GHC.Internal.System.Posix.Types GHC.Internal.TH.Lib \
        GHC.Internal.TH.Lift GHC.Internal.TH.Quote GHC.Internal.TH.Syntax \
        GHC.Internal.Text.ParserCombinators.ReadP GHC.Internal.Text.ParserCombinators.ReadPrec \
        GHC.Internal.Text.Read GHC.Internal.Text.Read.Lex GHC.Internal.Text.Show \
        GHC.Internal.TopHandler GHC.Internal.Tuple GHC.Internal.Type.Reflection \
        GHC.Internal.Type.Reflection.Unsafe GHC.Internal.TypeError GHC.Internal.TypeLits \
        GHC.Internal.TypeLits.Internal GHC.Internal.TypeNats GHC.Internal.TypeNats.Internal \
        GHC.Internal.Types GHC.Internal.Unicode GHC.Internal.Unsafe.Coerce GHC.Internal.Weak \
        GHC.Internal.Weak.Finalize GHC.Internal.Windows GHC.Internal.Word",
    ),
    (
        "ghc-platform",
        "\
        GHC.Platform.ArchOS",
    ),
    (
        "ghc-prim",
        "\
        GHC.CString GHC.Classes GHC.Debug GHC.Magic GHC.Magic.Dict GHC.Prim GHC.Prim.Exception \
        GHC.Prim.Ext GHC.Prim.Panic GHC.Prim.PtrEq GHC.PrimopWrappers GHC.Tuple GHC.Types",
    ),
    (
        "ghc-toolchain",
        "\
        GHC.Toolchain GHC.Toolchain.CheckArm GHC.Toolchain.Lens GHC.Toolchain.Monad \
        GHC.Toolchain.NormaliseTriple GHC.Toolchain.ParseTriple GHC.Toolchain.PlatformDetails \
        GHC.Toolchain.Prelude GHC.Toolchain.Program GHC.Toolchain.Target GHC.Toolchain.Tools.Ar \
        GHC.Toolchain.Tools.Cc GHC.Toolchain.Tools.Cpp GHC.Toolchain.Tools.Cxx \
        GHC.Toolchain.Tools.Link GHC.Toolchain.Tools.MergeObjs GHC.Toolchain.Tools.Nm \
        GHC.Toolchain.Tools.Ranlib GHC.Toolchain.Tools.Readelf GHC.Toolchain.Utils",
    ),
    (
        "ghci",
        "\
        GHCi.BinaryArray GHCi.BreakArray GHCi.CreateBCO GHCi.Debugger GHCi.FFI GHCi.InfoTable \
        GHCi.Message GHCi.ObjLink GHCi.RemoteTypes GHCi.ResolvedBCO GHCi.Run GHCi.Server \
        GHCi.Signals GHCi.StaticPtrTable GHCi.TH GHCi.TH.Binary GHCi.Utils",
    ),
    (
        "haddock-api",
        "\
        Documentation.Haddock",
    ),
    (
        "haddock-library",
        "\
        Documentation.Haddock.Doc Documentation.Haddock.Markup Documentation.Haddock.Parser \
        Documentation.Haddock.Types",
    ),
    (
        "haskeline",
        "\
        System.Console.Haskeline System.Console.Haskeline.Completion \
        System.Console.Haskeline.History System.Console.Haskeline.IO \
        System.Console.Haskeline.Internal",
    ),
    (
        "hpc",
        "\
        Trace.Hpc.Mix Trace.Hpc.Reflect Trace.Hpc.Tix Trace.Hpc.Util",
    ),
    (
        "integer-gmp",
        "\
        GHC.Integer.GMP.Internals",
    ),
    (
        "mtl",
        "\
        Control.Monad.Accum Control.Monad.Cont Control.Monad.Cont.Class \
        Control.Monad.Error.Class Control.Monad.Except Control.Monad.Identity Control.Monad.RWS \
        Control.Monad.RWS.CPS Control.Monad.RWS.Class Control.Monad.RWS.Lazy \
        Control.Monad.RWS.Strict Control.Monad.Reader Control.Monad.Reader.Class \
        Control.Monad.Select Control.Monad.State Control.Monad.State.Class \
        Control.Monad.State.Lazy Control.Monad.State.Strict Control.Monad.Trans \
        Control.Monad.Writer Control.Monad.Writer.CPS Control.Monad.Writer.Class \
        Control.Monad.Writer.Lazy Control.Monad.Writer.Strict",
    ),
    (
        "os-string",
        "\
        System.OsString System.OsString.Data.ByteString.Short \
        System.OsString.Data.ByteString.Short.Internal \
        System.OsString.Data.ByteString.Short.Word16 System.OsString.Encoding \
        System.OsString.Encoding.Internal System.OsString.Internal \
        System.OsString.Internal.Exception System.OsString.Internal.Types System.OsString.Posix \
        System.OsString.Windows",
    ),
    (
        "parsec",
        "\
        Text.Parsec Text.Parsec.ByteString Text.Parsec.ByteString.Lazy Text.Parsec.Char \
        Text.Parsec.Combinator Text.Parsec.Error Text.Parsec.Expr Text.Parsec.Language \
        Text.Parsec.Perm Text.Parsec.Pos Text.Parsec.Prim Text.Parsec.String Text.Parsec.Text \
        Text.Parsec.Text.Lazy Text.Parsec.Token Text.ParserCombinators.Parsec \
        Text.ParserCombinators.Parsec.Char Text.ParserCombinators.Parsec.Combinator \
        Text.ParserCombinators.Parsec.Error Text.ParserCombinators.Parsec.Expr \
        Text.ParserCombinators.Parsec.Language Text.ParserCombinators.Parsec.Perm \
        Text.ParserCombinators.Parsec.Pos Text.ParserCombinators.Parsec.Prim \
        Text.ParserCombinators.Parsec.Token",
    ),
    (
        "pretty",
        "\
        Text.PrettyPrint Text.PrettyPrint.Annotated Text.PrettyPrint.Annotated.HughesPJ \
        Text.PrettyPrint.Annotated.HughesPJClass Text.PrettyPrint.HughesPJ \
        Text.PrettyPrint.HughesPJClass",
    ),
    (
        "process",
        "\
        System.Cmd System.Process System.Process.CommunicationHandle \
        System.Process.CommunicationHandle.Internal System.Process.Environment.OsString \
        System.Process.Internals",
    ),
    (
        "semaphore-compat",
        "\
        System.Semaphore",
    ),
    (
        "stm",
        "\
        Control.Concurrent.STM Control.Concurrent.STM.TArray Control.Concurrent.STM.TBQueue \
        Control.Concurrent.STM.TChan Control.Concurrent.STM.TMVar Control.Concurrent.STM.TQueue \
        Control.Concurrent.STM.TSem Control.Concurrent.STM.TVar Control.Monad.STM",
    ),
    (
        "template-haskell",
        "\
        Language.Haskell.TH Language.Haskell.TH.CodeDo Language.Haskell.TH.LanguageExtensions \
        Language.Haskell.TH.Lib Language.Haskell.TH.Ppr Language.Haskell.TH.PprLib \
        Language.Haskell.TH.Quote Language.Haskell.TH.Syntax",
    ),
    (
        "template-haskell-lift",
        "\
        Language.Haskell.TH.Lift",
    ),
    (
        "template-haskell-quasiquoter",
        "\
        Language.Haskell.TH.QuasiQuoter",
    ),
    (
        "text",
        "\
        Data.Text Data.Text.Array Data.Text.Encoding Data.Text.Encoding.Error Data.Text.Foreign \
        Data.Text.IO Data.Text.IO.Utf8 Data.Text.Internal Data.Text.Internal.ArrayUtils \
        Data.Text.Internal.Builder Data.Text.Internal.Builder.Functions \
        Data.Text.Internal.Builder.Int.Digits Data.Text.Internal.Builder.RealFloat.Functions \
        Data.Text.Internal.ByteStringCompat Data.Text.Internal.Encoding \
        Data.Text.Internal.Encoding.Fusion Data.Text.Internal.Encoding.Fusion.Common \
        Data.Text.Internal.Encoding.Utf16 Data.Text.Internal.Encoding.Utf32 \
        Data.Text.Internal.Encoding.Utf8 Data.Text.Internal.Fusion \
        Data.Text.Internal.Fusion.CaseMapping Data.Text.Internal.Fusion.Common \
        Data.Text.Internal.Fusion.Size Data.Text.Internal.Fusion.Types Data.Text.Internal.IO \
        Data.Text.Internal.Lazy Data.Text.Internal.Lazy.Encoding.Fusion \
        Data.Text.Internal.Lazy.Fusion Data.Text.Internal.Lazy.Search \
        Data.Text.Internal.PrimCompat Data.Text.Internal.Private Data.Text.Internal.Read \
        Data.Text.Internal.Search Data.Text.Internal.StrictBuilder Data.Text.Internal.Unsafe \
        Data.Text.Internal.Unsafe.Char Data.Text.Internal.Validate \
        Data.Text.Internal.Validate.Native Data.Text.Lazy Data.Text.Lazy.Builder \
        Data.Text.Lazy.Builder.Int Data.Text.Lazy.Builder.RealFloat Data.Text.Lazy.Encoding \
        Data.Text.Lazy.IO Data.Text.Lazy.Internal Data.Text.Lazy.Read Data.Text.Read \
        Data.Text.Unsafe",
    ),
    (
        "time",
        "\
        Data.Time Data.Time.Calendar Data.Time.Calendar.Easter Data.Time.Calendar.Julian \
        Data.Time.Calendar.Month Data.Time.Calendar.MonthDay Data.Time.Calendar.OrdinalDate \
        Data.Time.Calendar.Quarter Data.Time.Calendar.WeekDate Data.Time.Clock \
        Data.Time.Clock.POSIX Data.Time.Clock.System Data.Time.Clock.TAI Data.Time.Format \
        Data.Time.Format.ISO8601 Data.Time.LocalTime",
    ),
    (
        "transformers",
        "\
        Control.Applicative.Backwards Control.Applicative.Lift Control.Monad.Signatures \
        Control.Monad.Trans.Accum Control.Monad.Trans.Class Control.Monad.Trans.Cont \
        Control.Monad.Trans.Except Control.Monad.Trans.Identity Control.Monad.Trans.Maybe \
        Control.Monad.Trans.RWS Control.Monad.Trans.RWS.CPS Control.Monad.Trans.RWS.Lazy \
        Control.Monad.Trans.RWS.Strict Control.Monad.Trans.Reader Control.Monad.Trans.Select \
        Control.Monad.Trans.State Control.Monad.Trans.State.Lazy \
        Control.Monad.Trans.State.Strict Control.Monad.Trans.Writer \
        Control.Monad.Trans.Writer.CPS Control.Monad.Trans.Writer.Lazy \
        Control.Monad.Trans.Writer.Strict Data.Functor.Constant Data.Functor.Reverse",
    ),
    (
        "xhtml",
        "\
        Text.XHtml Text.XHtml.Debug Text.XHtml.Frameset Text.XHtml.Strict Text.XHtml.Table \
        Text.XHtml.Transitional",
    ),
];
