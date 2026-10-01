-- | The ladders' name tables, part 1 of 2 (plan v2.32 step 1),
-- transcribed from cli/src/graph/ladder/java_jdk.rs,
-- cli/src/graph/ladder/ts_node.rs at a378e78c; from this commit on the
-- core is the authority.
module CE.Lang.Common.Ladder where

java :: String
java =
  "[ladder.java]\n\
  \# The Java ladder's External evidence (plan v2.30 step 3; booklet §8\n\
  \# row Java): what the JDK itself answers. Machine-generated\n\
  \# 2026-09-24 from Temurin 25.0.4.1+1 (OpenJDK 25.0.4.1+1-LTS) — the\n\
  \# CPython `sys.stdlib_module_names` / `go list std` / `ghc-pkg`\n\
  \# precedent, never hand-typed: PACKAGES is every package some module\n\
  \# of the runtime image exports unqualified (`java --list-modules`,\n\
  \# then `java --describe-module` per module, its `exports p` lines — a\n\
  \# qualified `exports p to m` is no API), and LANG is the public\n\
  \# top-level types of `java.lang`, read off the class files' own\n\
  \# access flags in the `lib/modules` image. A missing name degrades to\n\
  \# out_of_scope (precision-safe), visible in the ledger, and is repaid\n\
  \# by regenerating these lists from a newer JDK.\n\
  \# The 233 packages the JDK exports unqualified.\n\
  \packages = [\n\
  \  'com.sun.java.accessibility.util', 'com.sun.jdi', 'com.sun.jdi.connect',\n\
  \  'com.sun.jdi.connect.spi', 'com.sun.jdi.event', 'com.sun.jdi.request',\n\
  \  'com.sun.management', 'com.sun.net.httpserver', 'com.sun.net.httpserver.spi',\n\
  \  'com.sun.nio.file', 'com.sun.nio.sctp', 'com.sun.security.auth',\n\
  \  'com.sun.security.auth.callback', 'com.sun.security.auth.login',\n\
  \  'com.sun.security.auth.module', 'com.sun.security.jgss', 'com.sun.source.doctree',\n\
  \  'com.sun.source.tree', 'com.sun.source.util', 'com.sun.tools.attach',\n\
  \  'com.sun.tools.attach.spi', 'com.sun.tools.javac', 'com.sun.tools.jconsole',\n\
  \  'java.applet', 'java.awt', 'java.awt.color', 'java.awt.datatransfer',\n\
  \  'java.awt.desktop', 'java.awt.dnd', 'java.awt.event', 'java.awt.font', 'java.awt.geom',\n\
  \  'java.awt.im', 'java.awt.im.spi', 'java.awt.image', 'java.awt.image.renderable',\n\
  \  'java.awt.print', 'java.beans', 'java.beans.beancontext', 'java.io', 'java.lang',\n\
  \  'java.lang.annotation', 'java.lang.classfile', 'java.lang.classfile.attribute',\n\
  \  'java.lang.classfile.constantpool', 'java.lang.classfile.instruction',\n\
  \  'java.lang.constant', 'java.lang.foreign', 'java.lang.instrument', 'java.lang.invoke',\n\
  \  'java.lang.management', 'java.lang.module', 'java.lang.ref', 'java.lang.reflect',\n\
  \  'java.lang.runtime', 'java.math', 'java.net', 'java.net.http', 'java.net.spi',\n\
  \  'java.nio', 'java.nio.channels', 'java.nio.channels.spi', 'java.nio.charset',\n\
  \  'java.nio.charset.spi', 'java.nio.file', 'java.nio.file.attribute', 'java.nio.file.spi',\n\
  \  'java.rmi', 'java.rmi.dgc', 'java.rmi.registry', 'java.rmi.server', 'java.security',\n\
  \  'java.security.cert', 'java.security.interfaces', 'java.security.spec', 'java.sql',\n\
  \  'java.text', 'java.text.spi', 'java.time', 'java.time.chrono', 'java.time.format',\n\
  \  'java.time.temporal', 'java.time.zone', 'java.util', 'java.util.concurrent',\n\
  \  'java.util.concurrent.atomic', 'java.util.concurrent.locks', 'java.util.function',\n\
  \  'java.util.jar', 'java.util.logging', 'java.util.prefs', 'java.util.random',\n\
  \  'java.util.regex', 'java.util.spi', 'java.util.stream', 'java.util.zip',\n\
  \  'javax.accessibility', 'javax.annotation.processing', 'javax.crypto',\n"

java2 :: String
java2 =
  "  'javax.crypto.interfaces', 'javax.crypto.spec', 'javax.imageio', 'javax.imageio.event',\n\
  \  'javax.imageio.metadata', 'javax.imageio.plugins.bmp', 'javax.imageio.plugins.jpeg',\n\
  \  'javax.imageio.plugins.tiff', 'javax.imageio.spi', 'javax.imageio.stream',\n\
  \  'javax.lang.model', 'javax.lang.model.element', 'javax.lang.model.type',\n\
  \  'javax.lang.model.util', 'javax.management', 'javax.management.loading',\n\
  \  'javax.management.modelmbean', 'javax.management.monitor', 'javax.management.openmbean',\n\
  \  'javax.management.relation', 'javax.management.remote', 'javax.management.remote.rmi',\n\
  \  'javax.management.timer', 'javax.naming', 'javax.naming.directory',\n\
  \  'javax.naming.event', 'javax.naming.ldap', 'javax.naming.ldap.spi', 'javax.naming.spi',\n\
  \  'javax.net', 'javax.net.ssl', 'javax.print', 'javax.print.attribute',\n\
  \  'javax.print.attribute.standard', 'javax.print.event', 'javax.rmi.ssl', 'javax.script',\n\
  \  'javax.security.auth', 'javax.security.auth.callback', 'javax.security.auth.kerberos',\n\
  \  'javax.security.auth.login', 'javax.security.auth.spi', 'javax.security.auth.x500',\n\
  \  'javax.security.cert', 'javax.security.sasl', 'javax.smartcardio', 'javax.sound',\n\
  \  'javax.sound.midi', 'javax.sound.midi.spi', 'javax.sound.sampled',\n\
  \  'javax.sound.sampled.spi', 'javax.sql', 'javax.sql.rowset', 'javax.sql.rowset.serial',\n\
  \  'javax.sql.rowset.spi', 'javax.swing', 'javax.swing.border', 'javax.swing.colorchooser',\n\
  \  'javax.swing.event', 'javax.swing.filechooser', 'javax.swing.plaf',\n\
  \  'javax.swing.plaf.basic', 'javax.swing.plaf.metal', 'javax.swing.plaf.multi',\n\
  \  'javax.swing.plaf.nimbus', 'javax.swing.plaf.synth', 'javax.swing.table',\n\
  \  'javax.swing.text', 'javax.swing.text.html', 'javax.swing.text.html.parser',\n\
  \  'javax.swing.text.rtf', 'javax.swing.tree', 'javax.swing.undo', 'javax.tools',\n\
  \  'javax.transaction.xa', 'javax.xml', 'javax.xml.catalog', 'javax.xml.crypto',\n\
  \  'javax.xml.crypto.dom', 'javax.xml.crypto.dsig', 'javax.xml.crypto.dsig.dom',\n\
  \  'javax.xml.crypto.dsig.keyinfo', 'javax.xml.crypto.dsig.spec', 'javax.xml.datatype',\n\
  \  'javax.xml.namespace', 'javax.xml.parsers', 'javax.xml.stream',\n\
  \  'javax.xml.stream.events', 'javax.xml.stream.util', 'javax.xml.transform',\n\
  \  'javax.xml.transform.dom', 'javax.xml.transform.sax', 'javax.xml.transform.stax',\n\
  \  'javax.xml.transform.stream', 'javax.xml.validation', 'javax.xml.xpath', 'jdk.dynalink',\n\
  \  'jdk.dynalink.beans', 'jdk.dynalink.linker', 'jdk.dynalink.linker.support',\n\
  \  'jdk.dynalink.support', 'jdk.incubator.vector', 'jdk.javadoc.doclet', 'jdk.jfr',\n\
  \  'jdk.jfr.consumer', 'jdk.jshell', 'jdk.jshell.execution', 'jdk.jshell.spi',\n\
  \  'jdk.jshell.tool', 'jdk.management', 'jdk.management.jfr', 'jdk.net', 'jdk.nio',\n\
  \  'jdk.nio.mapmode', 'jdk.security.jarsigner', 'jdk.swing.interop', 'netscape.javascript',\n\
  \  'org.ietf.jgss', 'org.w3c.dom', 'org.w3c.dom.bootstrap', 'org.w3c.dom.css',\n\
  \  'org.w3c.dom.events', 'org.w3c.dom.html', 'org.w3c.dom.ls', 'org.w3c.dom.ranges',\n\
  \  'org.w3c.dom.stylesheets', 'org.w3c.dom.traversal', 'org.w3c.dom.views',\n\
  \  'org.w3c.dom.xpath', 'org.xml.sax', 'org.xml.sax.ext', 'org.xml.sax.helpers',\n\
  \  'sun.misc', 'sun.reflect',\n\
  \]\n"

java3 :: String
java3 =
  "# The 108 public top-level types of `java.lang` — the one\n\
  \# package every compilation unit imports implicitly (JLS 7.3).\n\
  \lang = [\n\
  \  'AbstractMethodError', 'Appendable', 'ArithmeticException',\n\
  \  'ArrayIndexOutOfBoundsException', 'ArrayStoreException', 'AssertionError',\n\
  \  'AutoCloseable', 'Boolean', 'BootstrapMethodError', 'Byte', 'CharSequence', 'Character',\n\
  \  'Class', 'ClassCastException', 'ClassCircularityError', 'ClassFormatError',\n\
  \  'ClassLoader', 'ClassNotFoundException', 'ClassValue', 'CloneNotSupportedException',\n\
  \  'Cloneable', 'Comparable', 'Deprecated', 'Double', 'Enum',\n\
  \  'EnumConstantNotPresentException', 'Error', 'Exception', 'ExceptionInInitializerError',\n\
  \  'Float', 'FunctionalInterface', 'IO', 'IllegalAccessError', 'IllegalAccessException',\n\
  \  'IllegalArgumentException', 'IllegalCallerException', 'IllegalMonitorStateException',\n\
  \  'IllegalStateException', 'IllegalThreadStateException', 'IncompatibleClassChangeError',\n\
  \  'IndexOutOfBoundsException', 'InheritableThreadLocal', 'InstantiationError',\n\
  \  'InstantiationException', 'Integer', 'InternalError', 'InterruptedException',\n\
  \  'Iterable', 'LayerInstantiationException', 'LinkageError', 'Long', 'MatchException',\n\
  \  'Math', 'Module', 'ModuleLayer', 'NegativeArraySizeException', 'NoClassDefFoundError',\n\
  \  'NoSuchFieldError', 'NoSuchFieldException', 'NoSuchMethodError',\n\
  \  'NoSuchMethodException', 'NullPointerException', 'Number', 'NumberFormatException',\n\
  \  'Object', 'OutOfMemoryError', 'Override', 'Package', 'Process', 'ProcessBuilder',\n\
  \  'ProcessHandle', 'Readable', 'Record', 'ReflectiveOperationException', 'Runnable',\n\
  \  'Runtime', 'RuntimeException', 'RuntimePermission', 'SafeVarargs', 'ScopedValue',\n\
  \  'SecurityException', 'SecurityManager', 'Short', 'StableValue', 'StackOverflowError',\n\
  \  'StackTraceElement', 'StackWalker', 'StrictMath', 'String', 'StringBuffer',\n\
  \  'StringBuilder', 'StringIndexOutOfBoundsException', 'SuppressWarnings', 'System',\n\
  \  'Thread', 'ThreadDeath', 'ThreadGroup', 'ThreadLocal', 'Throwable',\n\
  \  'TypeNotPresentException', 'UnknownError', 'UnsatisfiedLinkError',\n\
  \  'UnsupportedClassVersionError', 'UnsupportedOperationException', 'VerifyError',\n\
  \  'VirtualMachineError', 'Void', 'WrongThreadException',\n\
  \]\n"

ts :: String
ts =
  "[ladder.ts]\n\
  \# Node's builtin module names (plan v2.30 step 5b), machine-listed:\n\
  \# `require('module').builtinModules` of Node 22.22.1 — 68 names, each\n\
  \# also importable under the `node:` prefix — plus the four modules\n\
  \# that exist under the prefix alone (`node:sea`, `node:sqlite`,\n\
  \# `node:test`, `node:test/reporters`; each `require`d on the same Node\n\
  \# to confirm, 2026-09-26). A bare specifier naming one is External at\n\
  \# the bare rung whatever any package.json declares: Node serves a\n\
  \# builtin before any node_modules lookup (the resolver's\n\
  \# LOAD_NODE_MODULES step is never reached), and a `node:` name the\n\
  \# tables lack is nothing Node can load. Two space-separated literals,\n\
  \# not a tuple table.\n\
  \builtins = [\n\
  \  '_http_agent', '_http_client', '_http_common', '_http_incoming', '_http_outgoing',\n\
  \  '_http_server', '_stream_duplex', '_stream_passthrough', '_stream_readable',\n\
  \  '_stream_transform', '_stream_wrap', '_stream_writable', '_tls_common', '_tls_wrap',\n\
  \  'assert', 'assert/strict', 'async_hooks', 'buffer', 'child_process', 'cluster',\n\
  \  'console', 'constants', 'crypto', 'dgram', 'diagnostics_channel', 'dns', 'dns/promises',\n\
  \  'domain', 'events', 'fs', 'fs/promises', 'http', 'http2', 'https', 'inspector',\n\
  \  'inspector/promises', 'module', 'net', 'os', 'path', 'path/posix', 'path/win32',\n\
  \  'perf_hooks', 'process', 'punycode', 'querystring', 'readline', 'readline/promises',\n\
  \  'repl', 'stream', 'stream/consumers', 'stream/promises', 'stream/web', 'string_decoder',\n\
  \  'sys', 'timers', 'timers/promises', 'tls', 'trace_events', 'tty', 'url', 'util',\n\
  \  'util/types', 'v8', 'vm', 'wasi', 'worker_threads', 'zlib',\n\
  \]\n\
  \prefix_only = [\n\
  \  'sea', 'sqlite', 'test', 'test/reporters',\n\
  \]\n"
