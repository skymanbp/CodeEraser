-- | One invocation's explicit include chain, classified before the ladder
-- searches it (cli/src/graph/compdb_flags.rs until plan v2.33 W2-text;
-- each function below is the Rust function of the same name).
-- https://gcc.gnu.org/onlinedocs/gcc/Directory-Options.html, Directory
-- Options: `-iquote` applies "only to the quote form" and comes "before
-- all directories specified by -I"; `-isystem` follows -I, `-idirafter`
-- follows standard directories. `-I-`: earlier -I entries become
-- quote-only; the current file's directory is inhibited with "no way to
-- override this effect". `-iprefix` is concatenated literally: "you
-- should include the final '/'"; `-iwithprefixbefore` belongs with -I
-- and `-iwithprefix` with -idirafter. Installation defaults name no tree
-- path. For directory operands, "the '=' or $SYSROOT is replaced by the
-- sysroot prefix". If -I repeats a system directory, "the -I option is
-- ignored".
-- https://github.com/llvm/llvm-project/blob/main/clang/lib/Lex/InitHeaderSearch.cpp,
-- `RemoveDuplicates`: "drop the user dir... keeping the system dir"
-- across the angled and system groups; the quoted group is deduplicated
-- on its own.
-- https://gcc.gnu.org/onlinedocs/gcc/Darwin-Options.html, -F: frameworks
-- are "interleaved with those specified by -I options" in left-to-right
-- order. https://clang.llvm.org/docs/ClangCommandLineReference.html,
-- -F/-iframework: ordinary/system framework paths; -include/--include:
-- "Include file before parsing"; -imacros: "Include macros from file
-- before parsing". Forwarded -Xclang/-Xpreprocessor words are read;
-- other stages' operands are skipped.
-- https://learn.microsoft.com/en-us/cpp/preprocessor/hash-include-directive-c-cpp,
-- quoted lookup: own directory, open include stack in reverse, /I, then
-- INCLUDE; angle lookup omits the first two. The ladder supplies that
-- include stack.
-- https://learn.microsoft.com/en-us/cpp/build/reference/external-external-headers,
-- /external:I: "The space between /external:I and path is optional."
-- ClangCommandLineReference /imsvc adds paths "as if in %INCLUDE%", after
-- external directories. MSVC keeps first duplicates; GNU keeps the system
-- occurrence.
module CE.Resolve.Flags (Search (..), Chain (..), chain) where

import CE.Resolve.Cmdline (isMsvc)
import CE.Resolve.Tables (gnuFlags, skipFlags)
import Data.List (find, isPrefixOf, stripPrefix)
import qualified Data.Set as Set

-- | One searched directory: joined with the spelling, or a framework
-- directory (`A/B.h` as `A.framework/Headers/B.h`, then
-- `PrivateHeaders/B.h`).
data Search = SDir String | SFramework String
  deriving (Eq, Ord, Show)

-- | An invocation's placed directories: quoted-only, ordinary, then
-- system; the forced include and macro files as written.
data Chain = Chain
  { chMsvc :: Bool
  , chOwnDir :: Bool
  , chQuote :: [Search]
  , chBracket :: [Search]
  , chSystem :: [Search]
  , chForced :: [String]
  }
  deriving (Eq, Ord, Show)

-- | `chain`: read expanded argv, placing directories (the caller's
-- `place`) before cross-class duplicate removal.
chain :: [String] -> (String -> Maybe String) -> Chain
chain argv place = deduplicate (out {chSystem = chSystem out <> after})
 where
  msvc = maybe False isMsvc (take1 argv)
  opts = options argv msvc
  sysroot = case [v | (f, v) <- reverse opts, f `elem` ["--sysroot", "--sysroot=", "-isysroot"]] of
    (v : _) -> Just v
    [] -> Nothing
  (out, _, after) = foldl step (Chain msvc True [] [] [] [], Nothing, []) opts
  step (c, prefix, aft) (flag, value)
    | flag == "-I-" = (c {chOwnDir = False, chQuote = chQuote c <> chBracket c, chBracket = []}, prefix, aft)
    | flag == "-iprefix" = (c, Just value, aft)
    | flag `elem` ["-include", "--include", "--include=", "-imacros", "--imacros", "FI"] =
        (c {chForced = chForced c <> [value]}, prefix, aft)
    | otherwise = let (c', aft') = addDirectory c aft flag value prefix sysroot place in (c', prefix, aft')
  take1 (x : _) = Just x
  take1 [] = Nothing

-- | `forwarded`: forwarded front-end words are ordinary argv; unrelated
-- stages consume one word. The program (argv's first word) is skipped.
forwarded :: [String] -> Bool -> [String]
forwarded argv msvc = go (drop 1 argv)
 where
  go [] = []
  go (w : ws)
    | not msvc && w `elem` ["-Xclang", "-Xpreprocessor"] = go ws
    | not msvc && w `elem` ["-Xassembler", "-Xlinker", "-Xanalyzer"] = go (drop 1 ws)
    | otherwise = w : go ws

-- | `spelling`: one option spelling, leaving unknown arguments inert.
spelling :: String -> Bool -> Maybe String
spelling arg msvc
  | msvc = find (`isPrefixOf` arg) ["external:I", "imsvc", "FI", "Tc", "Tp", "I", "D", "U"]
  | arg == "-I-" || arg `elem` skipFlags = Just arg
  | otherwise = find (`isPrefixOf` arg) gnuFlags

-- | `options`: operand pairs in order; a recognized option without an
-- operand ends the scan.
options :: [String] -> Bool -> [(String, String)]
options argv msvc = go (forwarded argv msvc)
 where
  go [] = []
  go (word : ws) = case argOf word of
    Nothing -> go ws
    Just arg -> case spelling arg msvc of
      Nothing -> go ws
      Just "-I-" -> ("-I-", "") : go ws
      Just flag -> case drop (length flag) arg of
        "" -> case ws of
          v : ws' -> (flag, v) : go ws'
          [] -> []
        tl -> (flag, tl) : go ws
  argOf word
    | msvc = case word of
        c : rest | c == '/' || c == '-' -> Just rest
        _ -> Nothing
    | otherwise = Just word

-- | `add_directory`: route one directory into its class; the
-- after-system suffix is kept apart until the end.
addDirectory :: Chain -> [Search] -> String -> String -> Maybe String -> Maybe String -> (String -> Maybe String) -> (Chain, [Search])
addDirectory c aft flag value prefix sysroot place = case target of
  Nothing -> (c, aft)
  Just put -> case directory flag value prefix sysroot (chMsvc c) >>= place of
    Nothing -> (c, aft)
    Just dir -> put (if flag `elem` ["-F", "-iframework"] then SFramework dir else SDir dir)
 where
  target
    | flag == "-iquote" = Just (\s -> (c {chQuote = chQuote c <> [s]}, aft))
    | flag `elem` ["-I", "-F", "-iwithprefixbefore", "I"] = Just (\s -> (c {chBracket = chBracket c <> [s]}, aft))
    | flag `elem` ["-isystem", "-iframework", "external:I"] = Just (\s -> (c {chSystem = chSystem c <> [s]}, aft))
    | flag `elem` ["-idirafter", "-isystem-after", "-iwithprefix", "imsvc"] = Just (\s -> (c, aft <> [s]))
    | otherwise = Nothing

-- | `directory`: prefix concatenation and explicit sysroot substitution
-- precede the caller's placement.
directory :: String -> String -> Maybe String -> Maybe String -> Bool -> Maybe String
directory flag value prefix sysroot msvc
  | flag `elem` ["-iwithprefix", "-iwithprefixbefore"] = (<> value) <$> prefix
  | not msvc, Just tl <- stripPrefix "=" value <|> stripPrefix "$SYSROOT" value =
      if null tl
        then sysroot
        else (\s -> trimEnd s <> "/" <> dropWhile (== '/') tl) <$> sysroot
  | otherwise = Just value
 where
  trimEnd = reverse . dropWhile (== '/') . reverse
  a <|> b = maybe b Just a

-- | `deduplicate`: clang's `RemoveDuplicates` as `InitHeaderSearch::
-- Realize` calls it — the quoted group alone, then across the angled
-- and system groups, where a user directory a system directory repeats
-- is dropped (GNU); MSVC keeps first occurrences throughout.
deduplicate :: Chain -> Chain
deduplicate c = c {chQuote = quote, chBracket = bracket, chSystem = system}
 where
  (quote, _) = retain (const True) Set.empty (chQuote c)
  systems = Set.fromList (chSystem c)
  (bracket, seen) = retain (\e -> chMsvc c || not (Set.member e systems)) Set.empty (chBracket c)
  (system, _) = retain (const True) seen (chSystem c)
  retain keep = go
   where
    go seen' [] = ([], seen')
    go seen' (e : es)
      | keep e && not (Set.member e seen') = let (rest, s) = go (Set.insert e seen') es in (e : rest, s)
      | otherwise = go seen' es
