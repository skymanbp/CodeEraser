-- | The GHC global package table, part 1 of 3 (plan v2.32 step 1),
-- transcribed from cli/src/graph/ladder/hs_boot.rs at a378e78c; from this
-- commit on the core is the authority.
module CE.Lang.Common.Boot1 where

boot :: String
boot =
  "# GHC 9.14.1 global-package-db exposed modules — machine-generated\n\
  \# (2026-08-14) by scratchpad gen_hs_boot.py via `ghc-pkg list\n\
  \# --global` + `ghc-pkg field <pkg> exposed,exposed-modules`, never\n\
  \# hand-typed (the Go `go list std` / Python stdlib_module_names\n\
  \# precedent). Selection is the db's own facts, zero curation: every\n\
  \# global package except ghc (hidden); rts (no exposed modules); system-cxx-std-lib (no exposed modules) —\n\
  \# the hidden bit is the same visibility fact cabal honours.\n\
  \# Re-exports keep their EXPOSED name (`Name from pkg:Orig` → Name).\n\
  \# A missing name degrades to Unresolved (precision-safe), visible\n\
  \# in the ledger, and is repaid by regenerating this table. Sheer\n\
  \# LENGTH is the data, not style — the E01 file axis reads >300\n\
  \# here as a table fact with this header as its inline why.\n\
  \# (package name, its exposed modules, space-separated) — the\n\
  \# external rung's evidence base: 43 packages, 1371 modules.\n\
  \Cabal\n\
  \  Distribution.Backpack Distribution.Backpack.ComponentsGraph\n\
  \  Distribution.Backpack.Configure Distribution.Backpack.ConfiguredComponent\n\
  \  Distribution.Backpack.DescribeUnitId Distribution.Backpack.FullUnitId\n\
  \  Distribution.Backpack.LinkedComponent Distribution.Backpack.ModSubst\n\
  \  Distribution.Backpack.ModuleShape Distribution.Backpack.PreModuleShape\n\
  \  Distribution.CabalSpecVersion Distribution.Compat.Binary\n\
  \  Distribution.Compat.CharParsing Distribution.Compat.CreatePipe\n\
  \  Distribution.Compat.DList Distribution.Compat.Directory\n\
  \  Distribution.Compat.Environment Distribution.Compat.Exception\n\
  \  Distribution.Compat.FilePath Distribution.Compat.Graph\n\
  \  Distribution.Compat.Internal.TempFile Distribution.Compat.Lens\n\
  \  Distribution.Compat.MonadFail Distribution.Compat.Newtype\n\
  \  Distribution.Compat.NonEmptySet Distribution.Compat.Parsing\n\
  \  Distribution.Compat.Prelude Distribution.Compat.Prelude.Internal\n\
  \  Distribution.Compat.Process Distribution.Compat.ResponseFile\n\
  \  Distribution.Compat.Semigroup Distribution.Compat.Stack Distribution.Compat.Time\n\
  \  Distribution.Compiler Distribution.FieldGrammar Distribution.FieldGrammar.Class\n\
  \  Distribution.FieldGrammar.FieldDescrs Distribution.FieldGrammar.Newtypes\n\
  \  Distribution.FieldGrammar.Parsec Distribution.FieldGrammar.Pretty Distribution.Fields\n\
  \  Distribution.Fields.ConfVar Distribution.Fields.Field Distribution.Fields.Lexer\n\
  \  Distribution.Fields.LexerMonad Distribution.Fields.ParseResult\n\
  \  Distribution.Fields.Parser Distribution.Fields.Pretty\n\
  \  Distribution.InstalledPackageInfo Distribution.License Distribution.Make\n\
  \  Distribution.ModuleName Distribution.Package Distribution.PackageDescription\n\
  \  Distribution.PackageDescription.Check Distribution.PackageDescription.Configuration\n\
  \  Distribution.PackageDescription.FieldGrammar Distribution.PackageDescription.Parsec\n\
  \  Distribution.PackageDescription.PrettyPrint Distribution.PackageDescription.Quirks\n"

boot2 :: String
boot2 =
  "  Distribution.PackageDescription.Utils Distribution.Parsec Distribution.Parsec.Error\n\
  \  Distribution.Parsec.FieldLineStream Distribution.Parsec.Position\n\
  \  Distribution.Parsec.Warning Distribution.Pretty Distribution.ReadE Distribution.SPDX\n\
  \  Distribution.SPDX.License Distribution.SPDX.LicenseExceptionId\n\
  \  Distribution.SPDX.LicenseExpression Distribution.SPDX.LicenseId\n\
  \  Distribution.SPDX.LicenseListVersion Distribution.SPDX.LicenseReference\n\
  \  Distribution.Simple Distribution.Simple.Bench Distribution.Simple.Build\n\
  \  Distribution.Simple.Build.Inputs Distribution.Simple.Build.Macros\n\
  \  Distribution.Simple.Build.PackageInfoModule Distribution.Simple.Build.PathsModule\n\
  \  Distribution.Simple.BuildPaths Distribution.Simple.BuildTarget\n\
  \  Distribution.Simple.BuildToolDepends Distribution.Simple.BuildWay\n\
  \  Distribution.Simple.CCompiler Distribution.Simple.Command Distribution.Simple.Compiler\n\
  \  Distribution.Simple.Configure Distribution.Simple.Errors\n\
  \  Distribution.Simple.FileMonitor.Types Distribution.Simple.Flag Distribution.Simple.GHC\n\
  \  Distribution.Simple.GHCJS Distribution.Simple.Glob Distribution.Simple.Glob.Internal\n\
  \  Distribution.Simple.Haddock Distribution.Simple.Hpc Distribution.Simple.Install\n\
  \  Distribution.Simple.InstallDirs Distribution.Simple.InstallDirs.Internal\n\
  \  Distribution.Simple.LocalBuildInfo Distribution.Simple.PackageDescription\n\
  \  Distribution.Simple.PackageIndex Distribution.Simple.PreProcess\n\
  \  Distribution.Simple.PreProcess.Types Distribution.Simple.PreProcess.Unlit\n\
  \  Distribution.Simple.Program Distribution.Simple.Program.Ar\n\
  \  Distribution.Simple.Program.Builtin Distribution.Simple.Program.Db\n\
  \  Distribution.Simple.Program.Find Distribution.Simple.Program.GHC\n\
  \  Distribution.Simple.Program.HcPkg Distribution.Simple.Program.Hpc\n\
  \  Distribution.Simple.Program.Internal Distribution.Simple.Program.Ld\n\
  \  Distribution.Simple.Program.ResponseFile Distribution.Simple.Program.Run\n\
  \  Distribution.Simple.Program.Script Distribution.Simple.Program.Strip\n\
  \  Distribution.Simple.Program.Types Distribution.Simple.Register\n\
  \  Distribution.Simple.Setup Distribution.Simple.SetupHooks.Errors\n\
  \  Distribution.Simple.SetupHooks.Internal Distribution.Simple.SetupHooks.Rule\n\
  \  Distribution.Simple.ShowBuildInfo Distribution.Simple.SrcDist Distribution.Simple.Test\n\
  \  Distribution.Simple.Test.ExeV10 Distribution.Simple.Test.LibV09\n\
  \  Distribution.Simple.Test.Log Distribution.Simple.UHC Distribution.Simple.UserHooks\n\
  \  Distribution.Simple.Utils Distribution.System Distribution.TestSuite Distribution.Text\n\
  \  Distribution.Types.AbiDependency Distribution.Types.AbiHash\n\
  \  Distribution.Types.AnnotatedId Distribution.Types.Benchmark\n\
  \  Distribution.Types.Benchmark.Lens Distribution.Types.BenchmarkInterface\n\
  \  Distribution.Types.BenchmarkType Distribution.Types.BuildInfo\n\
  \  Distribution.Types.BuildInfo.Lens Distribution.Types.BuildType\n\
  \  Distribution.Types.Component Distribution.Types.ComponentId\n\
  \  Distribution.Types.ComponentInclude Distribution.Types.ComponentLocalBuildInfo\n\
  \  Distribution.Types.ComponentName Distribution.Types.ComponentRequestedSpec\n"

boot3 :: String
boot3 =
  "  Distribution.Types.CondTree Distribution.Types.Condition Distribution.Types.ConfVar\n\
  \  Distribution.Types.Dependency Distribution.Types.DependencyMap\n\
  \  Distribution.Types.DependencySatisfaction Distribution.Types.DumpBuildInfo\n\
  \  Distribution.Types.ExeDependency Distribution.Types.Executable\n\
  \  Distribution.Types.Executable.Lens Distribution.Types.ExecutableScope\n\
  \  Distribution.Types.ExposedModule Distribution.Types.Flag Distribution.Types.ForeignLib\n\
  \  Distribution.Types.ForeignLib.Lens Distribution.Types.ForeignLibOption\n\
  \  Distribution.Types.ForeignLibType Distribution.Types.GenericPackageDescription\n\
  \  Distribution.Types.GenericPackageDescription.Lens Distribution.Types.GivenComponent\n\
  \  Distribution.Types.HookedBuildInfo Distribution.Types.IncludeRenaming\n\
  \  Distribution.Types.InstalledPackageInfo\n\
  \  Distribution.Types.InstalledPackageInfo.FieldGrammar\n\
  \  Distribution.Types.InstalledPackageInfo.Lens Distribution.Types.LegacyExeDependency\n\
  \  Distribution.Types.Lens Distribution.Types.Library Distribution.Types.Library.Lens\n\
  \  Distribution.Types.LibraryName Distribution.Types.LibraryVisibility\n\
  \  Distribution.Types.LocalBuildConfig Distribution.Types.LocalBuildInfo\n\
  \  Distribution.Types.MissingDependency Distribution.Types.MissingDependencyReason\n\
  \  Distribution.Types.Mixin Distribution.Types.Module Distribution.Types.ModuleReexport\n\
  \  Distribution.Types.ModuleRenaming Distribution.Types.MungedPackageId\n\
  \  Distribution.Types.MungedPackageName Distribution.Types.PackageDescription\n\
  \  Distribution.Types.PackageDescription.Lens Distribution.Types.PackageId\n\
  \  Distribution.Types.PackageId.Lens Distribution.Types.PackageName\n\
  \  Distribution.Types.PackageName.Magic Distribution.Types.PackageVersionConstraint\n\
  \  Distribution.Types.ParStrat Distribution.Types.PkgconfigDependency\n\
  \  Distribution.Types.PkgconfigName Distribution.Types.PkgconfigVersion\n\
  \  Distribution.Types.PkgconfigVersionRange Distribution.Types.SetupBuildInfo\n\
  \  Distribution.Types.SetupBuildInfo.Lens Distribution.Types.SourceRepo\n\
  \  Distribution.Types.SourceRepo.Lens Distribution.Types.TargetInfo\n\
  \  Distribution.Types.TestSuite Distribution.Types.TestSuite.Lens\n\
  \  Distribution.Types.TestSuiteInterface Distribution.Types.TestType\n\
  \  Distribution.Types.UnitId Distribution.Types.UnqualComponentName\n\
  \  Distribution.Types.Version Distribution.Types.VersionInterval\n\
  \  Distribution.Types.VersionInterval.Legacy Distribution.Types.VersionRange\n\
  \  Distribution.Types.VersionRange.Internal Distribution.Utils.Base62\n\
  \  Distribution.Utils.Generic Distribution.Utils.IOData Distribution.Utils.Json\n\
  \  Distribution.Utils.LogProgress Distribution.Utils.MD5 Distribution.Utils.MapAccum\n\
  \  Distribution.Utils.NubList Distribution.Utils.Path Distribution.Utils.Progress\n\
  \  Distribution.Utils.ShortText Distribution.Utils.String Distribution.Utils.Structured\n\
  \  Distribution.Verbosity Distribution.Verbosity.Internal Distribution.Version\n\
  \  Language.Haskell.Extension\n"

boot4 :: String
boot4 =
  "Cabal-syntax\n\
  \  Distribution.Backpack Distribution.CabalSpecVersion Distribution.Compat.Binary\n\
  \  Distribution.Compat.CharParsing Distribution.Compat.DList\n\
  \  Distribution.Compat.Exception Distribution.Compat.Graph Distribution.Compat.Lens\n\
  \  Distribution.Compat.MonadFail Distribution.Compat.Newtype\n\
  \  Distribution.Compat.NonEmptySet Distribution.Compat.Parsing\n\
  \  Distribution.Compat.Prelude Distribution.Compat.Semigroup Distribution.Compiler\n\
  \  Distribution.FieldGrammar Distribution.FieldGrammar.Class\n\
  \  Distribution.FieldGrammar.FieldDescrs Distribution.FieldGrammar.Newtypes\n\
  \  Distribution.FieldGrammar.Parsec Distribution.FieldGrammar.Pretty Distribution.Fields\n\
  \  Distribution.Fields.ConfVar Distribution.Fields.Field Distribution.Fields.Lexer\n\
  \  Distribution.Fields.LexerMonad Distribution.Fields.ParseResult\n\
  \  Distribution.Fields.Parser Distribution.Fields.Pretty\n\
  \  Distribution.InstalledPackageInfo Distribution.License Distribution.ModuleName\n\
  \  Distribution.Package Distribution.PackageDescription\n\
  \  Distribution.PackageDescription.Configuration\n\
  \  Distribution.PackageDescription.FieldGrammar Distribution.PackageDescription.Parsec\n\
  \  Distribution.PackageDescription.PrettyPrint Distribution.PackageDescription.Quirks\n\
  \  Distribution.PackageDescription.Utils Distribution.Parsec Distribution.Parsec.Error\n\
  \  Distribution.Parsec.FieldLineStream Distribution.Parsec.Position\n\
  \  Distribution.Parsec.Warning Distribution.Pretty Distribution.SPDX\n\
  \  Distribution.SPDX.License Distribution.SPDX.LicenseExceptionId\n\
  \  Distribution.SPDX.LicenseExpression Distribution.SPDX.LicenseId\n\
  \  Distribution.SPDX.LicenseListVersion Distribution.SPDX.LicenseReference\n\
  \  Distribution.System Distribution.Text Distribution.Types.AbiDependency\n\
  \  Distribution.Types.AbiHash Distribution.Types.Benchmark\n\
  \  Distribution.Types.Benchmark.Lens Distribution.Types.BenchmarkInterface\n\
  \  Distribution.Types.BenchmarkType Distribution.Types.BuildInfo\n\
  \  Distribution.Types.BuildInfo.Lens Distribution.Types.BuildType\n\
  \  Distribution.Types.Component Distribution.Types.ComponentId\n\
  \  Distribution.Types.ComponentName Distribution.Types.ComponentRequestedSpec\n\
  \  Distribution.Types.CondTree Distribution.Types.Condition Distribution.Types.ConfVar\n\
  \  Distribution.Types.Dependency Distribution.Types.DependencyMap\n\
  \  Distribution.Types.DependencySatisfaction Distribution.Types.ExeDependency\n\
  \  Distribution.Types.Executable Distribution.Types.Executable.Lens\n\
  \  Distribution.Types.ExecutableScope Distribution.Types.ExposedModule\n\
  \  Distribution.Types.Flag Distribution.Types.ForeignLib\n\
  \  Distribution.Types.ForeignLib.Lens Distribution.Types.ForeignLibOption\n\
  \  Distribution.Types.ForeignLibType Distribution.Types.GenericPackageDescription\n\
  \  Distribution.Types.GenericPackageDescription.Lens Distribution.Types.HookedBuildInfo\n\
  \  Distribution.Types.IncludeRenaming Distribution.Types.InstalledPackageInfo\n\
  \  Distribution.Types.InstalledPackageInfo.FieldGrammar\n"

boot5 :: String
boot5 =
  "  Distribution.Types.InstalledPackageInfo.Lens Distribution.Types.LegacyExeDependency\n\
  \  Distribution.Types.Lens Distribution.Types.Library Distribution.Types.Library.Lens\n\
  \  Distribution.Types.LibraryName Distribution.Types.LibraryVisibility\n\
  \  Distribution.Types.MissingDependency Distribution.Types.MissingDependencyReason\n\
  \  Distribution.Types.Mixin Distribution.Types.Module Distribution.Types.ModuleReexport\n\
  \  Distribution.Types.ModuleRenaming Distribution.Types.MungedPackageId\n\
  \  Distribution.Types.MungedPackageName Distribution.Types.PackageDescription\n\
  \  Distribution.Types.PackageDescription.Lens Distribution.Types.PackageId\n\
  \  Distribution.Types.PackageId.Lens Distribution.Types.PackageName\n\
  \  Distribution.Types.PackageVersionConstraint Distribution.Types.PkgconfigDependency\n\
  \  Distribution.Types.PkgconfigName Distribution.Types.PkgconfigVersion\n\
  \  Distribution.Types.PkgconfigVersionRange Distribution.Types.SetupBuildInfo\n\
  \  Distribution.Types.SetupBuildInfo.Lens Distribution.Types.SourceRepo\n\
  \  Distribution.Types.SourceRepo.Lens Distribution.Types.TestSuite\n\
  \  Distribution.Types.TestSuite.Lens Distribution.Types.TestSuiteInterface\n\
  \  Distribution.Types.TestType Distribution.Types.UnitId\n\
  \  Distribution.Types.UnqualComponentName Distribution.Types.Version\n\
  \  Distribution.Types.VersionInterval Distribution.Types.VersionInterval.Legacy\n\
  \  Distribution.Types.VersionRange Distribution.Types.VersionRange.Internal\n\
  \  Distribution.Utils.Base62 Distribution.Utils.Generic Distribution.Utils.MD5\n\
  \  Distribution.Utils.Path Distribution.Utils.ShortText Distribution.Utils.String\n\
  \  Distribution.Utils.Structured Distribution.Version Language.Haskell.Extension\n"
