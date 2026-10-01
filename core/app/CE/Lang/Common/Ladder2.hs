-- | The ladders' name tables, part 2 of 2 (plan v2.32 step 1),
-- transcribed from cli/src/graph/ladder/go.rs, cli/src/graph/ladder/py.rs,
-- cli/src/graph/ladder/lua.rs, cli/src/graph/ladder/rs_use.rs at a378e78c;
-- from this commit on the core is the authority.
module CE.Lang.Common.Ladder2 where

go :: String
go =
  "[ladder.go]\n\
  \# Go 1.26.4 importable standard-library packages — machine-generated\n\
  \# (2026-08-13) via `go list std` with internal/ and vendor/ paths\n\
  \# filtered out (user code cannot import them), never hand-typed. A\n\
  \# missing name degrades to Unresolved (precision-safe), visible in\n\
  \# the ledger, and is repaid by regenerating this list (the one-shot\n\
  \# regen_tables drift check retired to git history).\n\
  \std = [\n\
  \  'archive/tar', 'archive/zip', 'bufio', 'bytes', 'cmp', 'compress/bzip2',\n\
  \  'compress/flate', 'compress/gzip', 'compress/lzw', 'compress/zlib', 'container/heap',\n\
  \  'container/list', 'container/ring', 'context', 'crypto', 'crypto/aes', 'crypto/cipher',\n\
  \  'crypto/des', 'crypto/dsa', 'crypto/ecdh', 'crypto/ecdsa', 'crypto/ed25519',\n\
  \  'crypto/elliptic', 'crypto/fips140', 'crypto/hkdf', 'crypto/hmac', 'crypto/hpke',\n\
  \  'crypto/md5', 'crypto/mlkem', 'crypto/mlkem/mlkemtest', 'crypto/pbkdf2', 'crypto/rand',\n\
  \  'crypto/rc4', 'crypto/rsa', 'crypto/sha1', 'crypto/sha256', 'crypto/sha3',\n\
  \  'crypto/sha512', 'crypto/subtle', 'crypto/tls', 'crypto/x509', 'crypto/x509/pkix',\n\
  \  'database/sql', 'database/sql/driver', 'debug/buildinfo', 'debug/dwarf', 'debug/elf',\n\
  \  'debug/gosym', 'debug/macho', 'debug/pe', 'debug/plan9obj', 'embed', 'encoding',\n\
  \  'encoding/ascii85', 'encoding/asn1', 'encoding/base32', 'encoding/base64',\n\
  \  'encoding/binary', 'encoding/csv', 'encoding/gob', 'encoding/hex', 'encoding/json',\n\
  \  'encoding/pem', 'encoding/xml', 'errors', 'expvar', 'flag', 'fmt', 'go/ast', 'go/build',\n\
  \  'go/build/constraint', 'go/constant', 'go/doc', 'go/doc/comment', 'go/format',\n\
  \  'go/importer', 'go/parser', 'go/printer', 'go/scanner', 'go/token', 'go/types',\n\
  \  'go/version', 'hash', 'hash/adler32', 'hash/crc32', 'hash/crc64', 'hash/fnv',\n\
  \  'hash/maphash', 'html', 'html/template', 'image', 'image/color', 'image/color/palette',\n\
  \  'image/draw', 'image/gif', 'image/jpeg', 'image/png', 'index/suffixarray', 'io',\n\
  \  'io/fs', 'io/ioutil', 'iter', 'log', 'log/slog', 'log/syslog', 'maps', 'math',\n\
  \  'math/big', 'math/bits', 'math/cmplx', 'math/rand', 'math/rand/v2', 'mime',\n\
  \  'mime/multipart', 'mime/quotedprintable', 'net', 'net/http', 'net/http/cgi',\n\
  \  'net/http/cookiejar', 'net/http/fcgi', 'net/http/httptest', 'net/http/httptrace',\n\
  \  'net/http/httputil', 'net/http/pprof', 'net/mail', 'net/netip', 'net/rpc',\n\
  \  'net/rpc/jsonrpc', 'net/smtp', 'net/textproto', 'net/url', 'os', 'os/exec', 'os/signal',\n\
  \  'os/user', 'path', 'path/filepath', 'plugin', 'reflect', 'regexp', 'regexp/syntax',\n\
  \  'runtime', 'runtime/cgo', 'runtime/coverage', 'runtime/debug', 'runtime/metrics',\n\
  \  'runtime/pprof', 'runtime/race', 'runtime/trace', 'slices', 'sort', 'strconv',\n\
  \  'strings', 'structs', 'sync', 'sync/atomic', 'syscall', 'testing', 'testing/cryptotest',\n\
  \  'testing/fstest', 'testing/iotest', 'testing/quick', 'testing/slogtest',\n\
  \  'testing/synctest', 'text/scanner', 'text/tabwriter', 'text/template',\n\
  \  'text/template/parse', 'time', 'time/tzdata', 'unicode', 'unicode/utf16',\n\
  \  'unicode/utf8', 'unique', 'unsafe', 'weak',\n\
  \]\n"

py :: String
py =
  "[ladder.py]\n\
  \# CPython 3.13 public top-level stdlib modules — machine-generated\n\
  \# from `sys.stdlib_module_names` (2026-08-12), never hand-typed.\n\
  \# A missing name degrades to Unresolved (precision-safe), visible\n\
  \# in the ledger, and is repaid by regenerating this list (the\n\
  \# one-shot regen_tables drift check retired to git history).\n\
  \stdlib = [\n\
  \  'abc', 'antigravity', 'argparse', 'array', 'ast', 'asyncio', 'atexit', 'base64', 'bdb',\n\
  \  'binascii', 'bisect', 'builtins', 'bz2', 'cProfile', 'calendar', 'cmath', 'cmd', 'code',\n\
  \  'codecs', 'codeop', 'collections', 'colorsys', 'compileall', 'concurrent',\n\
  \  'configparser', 'contextlib', 'contextvars', 'copy', 'copyreg', 'csv', 'ctypes',\n\
  \  'curses', 'dataclasses', 'datetime', 'dbm', 'decimal', 'difflib', 'dis', 'doctest',\n\
  \  'email', 'encodings', 'ensurepip', 'enum', 'errno', 'faulthandler', 'fcntl', 'filecmp',\n\
  \  'fileinput', 'fnmatch', 'fractions', 'ftplib', 'functools', 'gc', 'genericpath',\n\
  \  'getopt', 'getpass', 'gettext', 'glob', 'graphlib', 'grp', 'gzip', 'hashlib', 'heapq',\n\
  \  'hmac', 'html', 'http', 'idlelib', 'imaplib', 'importlib', 'inspect', 'io', 'ipaddress',\n\
  \  'itertools', 'json', 'keyword', 'linecache', 'locale', 'logging', 'lzma', 'mailbox',\n\
  \  'marshal', 'math', 'mimetypes', 'mmap', 'modulefinder', 'msvcrt', 'multiprocessing',\n\
  \  'netrc', 'nt', 'ntpath', 'nturl2path', 'numbers', 'opcode', 'operator', 'optparse',\n\
  \  'os', 'pathlib', 'pdb', 'pickle', 'pickletools', 'pkgutil', 'platform', 'plistlib',\n\
  \  'poplib', 'posix', 'posixpath', 'pprint', 'profile', 'pstats', 'pty', 'pwd',\n\
  \  'py_compile', 'pyclbr', 'pydoc', 'pydoc_data', 'pyexpat', 'queue', 'quopri', 'random',\n\
  \  're', 'readline', 'reprlib', 'resource', 'rlcompleter', 'runpy', 'sched', 'secrets',\n\
  \  'select', 'selectors', 'shelve', 'shlex', 'shutil', 'signal', 'site', 'smtplib',\n\
  \  'socket', 'socketserver', 'sqlite3', 'sre_compile', 'sre_constants', 'sre_parse', 'ssl',\n\
  \  'stat', 'statistics', 'string', 'stringprep', 'struct', 'subprocess', 'symtable', 'sys',\n\
  \  'sysconfig', 'syslog', 'tabnanny', 'tarfile', 'tempfile', 'termios', 'textwrap', 'this',\n\
  \  'threading', 'time', 'timeit', 'tkinter', 'token', 'tokenize', 'tomllib', 'trace',\n\
  \  'traceback', 'tracemalloc', 'tty', 'turtle', 'turtledemo', 'types', 'typing',\n\
  \  'unicodedata', 'unittest', 'urllib', 'uuid', 'venv', 'warnings', 'wave', 'weakref',\n\
  \  'webbrowser', 'winreg', 'winsound', 'wsgiref', 'xml', 'xmlrpc', 'zipapp', 'zipfile',\n\
  \  'zipimport', 'zlib', 'zoneinfo',\n\
  \]\n"

lua :: String
lua =
  "[ladder.lua]\n\
  \# The names `package.loaded` holds before any searcher runs: Lua\n\
  \# 5.1–5.4's standard libraries (manual §6; `bit32` is 5.2's, `utf8`\n\
  \# 5.3's) and LuaJIT's built-in extension modules\n\
  \# (luajit.org/extensions.html).\n\
  \stdlib = [\n\
  \  'string', 'table', 'math', 'io', 'os', 'coroutine', 'debug', 'package', 'bit32', 'utf8',\n\
  \  'ffi', 'bit', 'jit', 'jit.util', 'jit.profile', 'table.new', 'table.clear',\n\
  \  'string.buffer',\n\
  \]\n"

rs :: String
rs =
  "[ladder.rs]\n\
  \# Crates the toolchain provides without any declaration.\n\
  \builtin = [\n\
  \  'std', 'core', 'alloc', 'proc_macro', 'test',\n\
  \]\n"
