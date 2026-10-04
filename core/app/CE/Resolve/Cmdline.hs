-- | Compiler command and response-file argv, read without consulting the
-- host (cli/src/graph/cmdline.rs until plan v2.33 W2-text; each function
-- below is the Rust function of the same name, the character loop of the
-- Rust `Peekable<Chars>` spelled as a list the function returns the rest
-- of). Clang's JSON reader chooses its syntax with the host's `_WIN32`;
-- here the command's own program token chooses it so one database reads
-- alike everywhere.
-- https://github.com/llvm/llvm-project/blob/main/clang/lib/Tooling/JSONCompilationDatabase.cpp
-- `CommandLineArgumentParser`: only space separates; adjacent quoted and
-- free pieces concatenate, single quotes are literal, and a partial final
-- word stays.
-- https://github.com/llvm/llvm-project/blob/main/llvm/lib/Support/CommandLine.cpp
-- `TokenizeGNUCommandLine`: "Consume runs of whitespace", "Backslash
-- escapes the next character", including inside either kind of quote. A
-- final slash stays. `TokenizeWindowsCommandLine`, `parseBackslash`:
-- pairs of slashes before a quote yield slashes; an odd remainder
-- escapes the quote. "Consecutive double-quotes inside a quoted string
-- implies one double-quote". The leading program follows CreateProcess:
-- its backslashes are literal. These are lexical readers, not shells.
module CE.Resolve.Cmdline (
  windowsShaped,
  splitCommand,
  splitGnuJson,
  splitGnu,
  splitWindows,
  isMsvc,
) where

import Data.Char (isAsciiLower, isAsciiUpper, toLower)
import Data.List (isSuffixOf)

-- | `compdb_flags::is_msvc`: an MSVC driver basename under either
-- separator, compared without ASCII case.
isMsvc :: String -> Bool
isMsvc prog = any (eqIgnoreAsciiCase name) ["cl", "cl.exe", "clang-cl", "clang-cl.exe"]
 where
  name = reverse (takeWhile (`notElem` ("/\\" :: String)) (reverse prog))

-- | `str::eq_ignore_ascii_case`.
eqIgnoreAsciiCase :: String -> String -> Bool
eqIgnoreAsciiCase a b = length a == length b && and (zipWith (\x y -> asciiLower x == asciiLower y) a b)

asciiLower :: Char -> Char
asciiLower c = if isAsciiUpper c then toLower c else c

-- | A Windows tool spelling: separator, drive, executable suffix, or MSVC
-- basename.
windowsShaped :: String -> Bool
windowsShaped prog = '\\' `elem` prog || drive || any (`isSuffixOf` map asciiLower prog) [".exe", ".bat", ".cmd"] || isMsvc prog
 where
  drive = case prog of
    (c : ':' : _) -> isAsciiUpper c || isAsciiLower c
    _ -> False

-- | `program_of`: the command's first space-delimited word, or its
-- initial double-quoted run.
programOf :: String -> String
programOf command = case dropWhile (== ' ') command of
  '"' : rest -> takeWhile (/= '"') rest
  c -> takeWhile (/= ' ') c

-- | A database command in the syntax its program spelling selects.
splitCommand :: String -> [String]
splitCommand command
  | windowsShaped (programOf command) = splitWindows command True
  | otherwise = splitGnuJson command

-- | Clang's JSON POSIX command reader: only space separates, single
-- quotes are literal.
splitGnuJson :: String -> [String]
splitGnuJson command = tokenize (== ' ') (const (unixWord True)) command

-- | LLVM's GNU response reader: ASCII whitespace separates, both quotes
-- allow escapes.
splitGnu :: String -> [String]
splitGnu text = tokenize whitespace (const (unixWord False)) text

-- | LLVM's Windows response reader, optionally with CreateProcess
-- program handling for the first word.
splitWindows :: String -> Bool -> [String]
splitWindows text leadingProgram = tokenize windowsSpace word text
 where
  word first = if first && leadingProgram then program else windowsWord

-- | The common word loop: separators skipped, then one word; a word is
-- told whether it is the first.
tokenize :: (Char -> Bool) -> (Bool -> String -> (String, String)) -> String -> [String]
tokenize separator word = go True
 where
  go first input = case dropWhile separator input of
    [] -> []
    rest -> let (out, rest') = word first rest in out : go False rest'

-- | LLVM's four whitespace characters.
whitespace :: Char -> Bool
whitespace c = c `elem` (" \t\r\n" :: String)

-- | Windows also accepts NUL between arguments.
windowsSpace :: Char -> Bool
windowsSpace c = whitespace c || c == '\0'

-- | `unix_word`: adjacent free and quoted pieces of either GNU reader.
unixWord :: Bool -> String -> (String, String)
unixWord json = go ""
 where
  separator = if json then (== ' ') else whitespace
  go acc input = case input of
    c : rest | not (separator c) -> case c of
      _ | c == '"' || c == '\'' -> let (piece, rest') = quoted json c rest in go (reverse piece <> acc) rest'
      '\\' -> let (piece, rest') = escaped json rest in go (reverse piece <> acc) rest'
      _ -> go (c : acc) rest
    _ -> (reverse acc, input)

-- | `quoted`: one GNU quoted piece, with JSON single quotes disabling
-- backslash escapes; an unclosed quote runs to the end.
quoted :: Bool -> Char -> String -> (String, String)
quoted json quote = go ""
 where
  go acc input = case input of
    [] -> (reverse acc, [])
    c : rest
      | c == quote -> (reverse acc, rest)
      | c == '\\' && (not json || quote == '"') -> let (piece, rest') = escaped json rest in go (reverse piece <> acc) rest'
      | otherwise -> go (c : acc) rest

-- | `escaped`: an escape's following character; only the response
-- reader keeps a terminal slash.
escaped :: Bool -> String -> (String, String)
escaped json input = case input of
  c : rest -> ([c], rest)
  [] -> (if json then "" else "\\", [])

-- | `program`: a leading Windows program ends at whitespace or its
-- opening quote's mate.
program :: String -> (String, String)
program input = case input of
  '"' : rest -> go True "" rest
  _ -> go False "" input
 where
  go isQuoted acc s = case s of
    c : rest
      | isQuoted || not (windowsSpace c) ->
          if isQuoted && c == '"' then (reverse acc, rest) else go isQuoted (c : acc) rest
    _ -> (reverse acc, s)

-- | `windows_word`: a Windows argument, whose quoted runs may contain
-- doubled literal quotes.
windowsWord :: String -> (String, String)
windowsWord = go False ""
 where
  go isQuoted acc s = case s of
    c : rest
      | isQuoted || not (windowsSpace c) -> case c of
          '\\' -> let (piece, rest') = backslashes rest in go isQuoted (reverse piece <> acc) rest'
          '"'
            | isQuoted, '"' : rest' <- rest -> go isQuoted ('"' : acc) rest'
            | otherwise -> go (not isQuoted) acc rest
          _ -> go isQuoted (c : acc) rest
    _ -> (reverse acc, s)

-- | `backslashes`: a slash run with its first slash consumed; only a
-- following quote changes it.
backslashes :: String -> (String, String)
backslashes input = case after of
  '"' : rest
    | odd count -> (replicate (count `div` 2) '\\' <> "\"", rest)
    | otherwise -> (replicate (count `div` 2) '\\', after)
  _ -> (replicate count '\\', after)
 where
  (run, after) = span (== '\\') input
  count = 1 + length run
