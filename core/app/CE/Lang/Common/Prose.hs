-- | The prose definition tables (plan v2.32 step 1),
-- transcribed from cli/src/tombstone/vocab.rs, cli/src/docdup/spec.rs at
-- a378e78c; from this commit on the core is the authority.
module CE.Lang.Common.Prose where

tombstone :: String
tombstone =
  "[tombstone]\n\
  \# V₀ — absence words that are never names (lower-cased; the wide\n\
  \# four verbatim).\n\
  \negations = [\n\
  \  'no', 'not', 'non', 'none', 'null', 'nil', 'nan', 'noop', 'nop', 'nonzero', 'nonnull',\n\
  \  'notnull', 'nostd', 'notfound', 'notimplemented', 'nosuch', 'without', 'unless',\n\
  \  'never', 'false', 'off', 'disabled', 'empty', 'void', 'unset', 'missing', 'absent', '无',\n\
  \  '非', '否', '空',\n\
  \]\n\
  \# Reserved words of the six judged languages, sorted, lower-cased: a\n\
  \# name made of these alone is syntax, not a name.\n\
  \keywords = [\n\
  \  'abstract', 'and', 'any', 'assert', 'async', 'await', 'become', 'boolean', 'box',\n\
  \  'break', 'case', 'catch', 'chan', 'class', 'const', 'constructor', 'continue', 'crate',\n\
  \  'data', 'debugger', 'declare', 'def', 'default', 'defer', 'del', 'delete', 'deriving',\n\
  \  'do', 'dyn', 'elif', 'else', 'enum', 'except', 'export', 'extends', 'extern',\n\
  \  'fallthrough', 'finally', 'fn', 'for', 'forall', 'foreign', 'from', 'func', 'function',\n\
  \  'get', 'global', 'go', 'goto', 'hiding', 'if', 'impl', 'implements', 'import', 'in',\n\
  \  'infix', 'infixl', 'infixr', 'instance', 'instanceof', 'interface', 'iota', 'is',\n\
  \  'keyof', 'lambda', 'let', 'loop', 'macro', 'map', 'match', 'mod', 'module', 'move',\n\
  \  'mut', 'namespace', 'new', 'newtype', 'nonlocal', 'number', 'object', 'of', 'or',\n\
  \  'package', 'pass', 'private', 'protected', 'pub', 'public', 'qualified', 'raise',\n\
  \  'range', 'readonly', 'ref', 'require', 'return', 'select', 'self', 'set', 'static',\n\
  \  'string', 'struct', 'super', 'switch', 'symbol', 'then', 'this', 'throw', 'trait',\n\
  \  'true', 'try', 'type', 'typeof', 'undefined', 'union', 'unique', 'unknown', 'unsafe',\n\
  \  'use', 'var', 'where', 'while', 'with', 'yield',\n\
  \]\n\
  \# English absence frames: a prefix binds the words after it, a\n\
  \# suffix the words before it.\n\
  \en_prefix = [\n\
  \  'no', 'not', 'non', 'without', 'sans', 'minus',\n\
  \]\n\
  \en_suffix = [\n\
  \  'free', 'less', 'removed', 'dropped', 'gone',\n\
  \]\n\
  \# Chinese absence frames, read inside one run (`无东坡肉`) or as a\n\
  \# word before an ASCII name (`无cache`).\n\
  \zh_prefix = [\n\
  \  '无', '非', '不含', '不带', '没有', '去掉', '去', '免', '已删',\n\
  \]\n"

tombstone2 :: String
tombstone2 =
  "zh_suffix = [\n\
  \  '已删', '已移除', '已去掉', '不再有',\n\
  \]\n\
  \# Retrospective marks a prose segment carries: lower-cased English\n\
  \# phrases matched at word boundaries, Chinese by substring.\n\
  \marks_en = [\n\
  \  'no longer', 'previously', 'formerly', 'used to', 'we removed', 'was removed',\n\
  \  'has been removed', 'is no longer needed', 'is not needed here', 'is unnecessary',\n\
  \  'there is no need to', 'deliberately omitted', 'intentionally absent',\n\
  \  'rather than adding',\n\
  \]\n\
  \marks_zh = [\n\
  \  '不再', '此前', '原先', '曾经', '已去掉', '已删除', '已移除', '不需要', '没有必要', '无需', '故意不', '刻意不加', '之所以不',\n\
  \]\n\
  \# English function words: a window holding one straddles a phrase\n\
  \# boundary (`the_pre`, `budget_is`, `self_and_nth` — sentence-shaped\n\
  \# test names cut into windows) and is no name. Read only through\n\
  \# `vocabulary` (and the test that walks every table).\n\
  \stop_en = [\n\
  \  'a', 'an', 'the', 'is', 'are', 'was', 'were', 'be', 'been', 'being', 'am', 'of', 'to',\n\
  \  'in', 'on', 'at', 'by', 'for', 'with', 'as', 'it', 'its', 'this', 'that', 'these',\n\
  \  'those', 'than', 'then', 'and', 'or', 'but', 'so', 'if', 'do', 'does', 'did', 'has',\n\
  \  'have', 'had', 'from', 'into', 'onto', 'over', 'under', 'up', 'down', 'out', 'off',\n\
  \  'all', 'any', 'each', 'per', 'via', 'vs', 'we', 'you', 'they', 'our', 'your', 'their',\n\
  \  'my', 'me', 'us', 'him', 'her', 'his', 'who', 'what', 'which', 'when', 'where', 'why',\n\
  \  'how', 'here', 'there', 'also', 'only', 'just', 'very', 'still', 'yet', 'too', 'both',\n\
  \  'either', 'neither', 'such', 'same', 'other', 'another', 'should', 'would', 'could',\n\
  \  'can', 'may', 'might', 'must', 'will', 'shall',\n\
  \]\n\
  \# Fewest chars an ASCII name has (`ab` is a preposition, not a name).\n\
  \min_ascii_name = 3\n\
  \# Fewest chars a wide (CJK) name has.\n\
  \min_wide_name = 2\n\
  \# Widest window of adjacent words one spelling covers.\n\
  \join_max = 3\n\
  \# Bracket pairs (ASCII and full-width) that make a frame `bracketed`.\n\
  \open = ['(', '（', '[', '【']\n\
  \close = [')', '）', ']', '】']\n"

docdup :: String
docdup =
  "[docdup]\n\
  \# License-header markers (design vol.2 §5.2), `|`-separated. Any one\n\
  \# on any line of the first comment block inside the head window\n\
  \# exempts the block. Both marker tables are one literal rather than\n\
  \# an array: a run of string literals is one repeated token under the\n\
  \# clone gate.\n\
  \license_markers = [\n\
  \  'SPDX-License-Identifier', 'Licensed under the Apache License', 'Copyright (c)',\n\
  \  'Permission is hereby granted', 'MIT License',\n\
  \]\n\
  \# Structured-docstring skeleton line prefixes, `|`-separated (plan\n\
  \# :79 \"template rows\", stripped line-level from comment/docstring\n\
  \# segments): the Google/Sphinx/NumPy/JSDoc section vocabulary, not\n\
  \# prose, and since plan v2.30 step 5 the Javadoc / Doxygen / roxygen /\n\
  \# LDoc tags (booklet §9) — `@return` covers `@returns` and `@throw`\n\
  \# covers `@throws` (a prefix), `@exception` is Javadoc's synonym, and\n\
  \# a Doxygen command reads the same under `\\` as under `@`.\n\
  \skeleton_prefixes = [\n\
  \  'Args:', 'Arguments:', 'Returns:', 'Raises:', 'Yields:', 'Parameters', 'Attributes:',\n\
  \  'Example:', 'Examples:', 'Note:', ':param ', ':return', ':rtype', '@param', '@return',\n\
  \  '@throw', '@brief', '@see', '@since', '@author', '@version', '@exception', '@tparam',\n\
  \  '@treturn', '@usage', '@examples', '@export', '@importFrom', '@rdname', '@details',\n\
  \  '@inheritParams', '@describeIn', \"\\\\brief\", \"\\\\details\", \"\\\\param\", \"\\\\tparam\",\n\
  \  \"\\\\return\", \"\\\\throw\", \"\\\\exception\", \"\\\\see\", \"\\\\since\", \"\\\\author\", \"\\\\version\",\n\
  \]\n\
  \# The inline exemption marker (plan :79-80). Without a ` -- <why>`\n\
  \# tail it exempts NOTHING — a bare marker is a violation, ledgered.\n\
  \allow_marker = 'ce:allow(docdup)'\n\
  \# Segment kinds as frozen position codes (the wire.rs edge-code\n\
  \# discipline: reordering is a DOCDUP_REV bump; `html_text` — a block\n\
  \# element's prose, docdup/html.rs — appended at rev 6, plan v2.30\n\
  \# step 5; `text_para` — a plain-text file's paragraph, segments.rs\n\
  \# text_paragraphs — at rev 8, step 5b-8).\n\
  \kind_names = [\n\
  \  'md_para', 'comment_block', 'docstring', 'html_text', 'text_para',\n\
  \]\n"
