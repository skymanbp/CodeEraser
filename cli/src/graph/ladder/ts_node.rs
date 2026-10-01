//! Node's builtin module names (plan v2.30 step 5b), machine-listed:
//! `require('module').builtinModules` of Node 22.22.1 — 68 names, each
//! also importable under the `node:` prefix — plus the four modules
//! that exist under the prefix alone (`node:sea`, `node:sqlite`,
//! `node:test`, `node:test/reporters`; each `require`d on the same Node
//! to confirm, 2026-09-26). A bare specifier naming one is External at
//! the bare rung whatever any package.json declares: Node serves a
//! builtin before any node_modules lookup (the resolver's
//! LOAD_NODE_MODULES step is never reached), and a `node:` name the
//! tables lack is nothing Node can load. Two space-separated literals,
//! not a tuple table.

pub(crate) const BUILTINS: &str = "_http_agent _http_client _http_common _http_incoming _http_outgoing \
    _http_server _stream_duplex _stream_passthrough _stream_readable _stream_transform \
    _stream_wrap _stream_writable _tls_common _tls_wrap assert assert/strict async_hooks \
    buffer child_process cluster console constants crypto dgram diagnostics_channel dns \
    dns/promises domain events fs fs/promises http http2 https inspector inspector/promises \
    module net os path path/posix path/win32 perf_hooks process punycode querystring readline \
    readline/promises repl stream stream/consumers stream/promises stream/web string_decoder \
    sys timers timers/promises tls trace_events tty url util util/types v8 vm wasi \
    worker_threads zlib";

pub(crate) const PREFIX_ONLY: &str = "sea sqlite test test/reporters";

/// Whether `spec` names a Node builtin: the bare name from the first
/// table, or `node:` before a name from either.
pub(super) fn is_builtin(spec: &str) -> bool {
    let listed = |table: &str, name: &str| table.split_whitespace().any(|m| m == name);
    match spec.strip_prefix("node:") {
        Some(name) => listed(BUILTINS, name) || listed(PREFIX_ONLY, name),
        None => listed(BUILTINS, spec),
    }
}
