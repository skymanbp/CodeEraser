// A minimal Chrome DevTools Protocol client, over the WebSocket Node
// has had built in since v22 — which is what lets its callers run in a
// clean checkout with nothing installed. Split from shoot_gui.js at the
// repo's own 300-line file gate; driving a browser and photographing
// the GUI are two jobs, and only the second is about CodeEraser.

"use strict";

const fs = require("fs");
const path = require("path");

/// One attached page. `send` is the raw protocol; `eval` and `until`
/// are the two conveniences every caller here needs.
class Devtools {
  constructor(ws) {
    this.ws = ws;
    this.id = 0;
    this.waiting = new Map();
    ws.addEventListener("message", (e) => {
      const msg = JSON.parse(e.data);
      const slot = this.waiting.get(msg.id);
      if (!slot) return;
      this.waiting.delete(msg.id);
      msg.error ? slot.bad(new Error(msg.error.message)) : slot.ok(msg.result);
    });
    // A browser that dies mid-run would otherwise leave every pending
    // call unsettled forever: the caller awaits a reply that cannot
    // come, no cleanup runs, and the temp profile leaks. A hang is the
    // harder failure to read, so the socket closing is an ANSWER.
    const abandon = () => {
      const pending = [...this.waiting.values()];
      this.waiting.clear();
      for (const slot of pending) slot.bad(new Error("the browser went away mid-run"));
    };
    ws.addEventListener("close", abandon);
    ws.addEventListener("error", abandon);
  }

  static async open(url) {
    const ws = new WebSocket(url);
    await new Promise((ok, bad) => {
      ws.addEventListener("open", ok, { once: true });
      ws.addEventListener("error", () => bad(new Error(`cannot reach ${url}`)), { once: true });
    });
    return new Devtools(ws);
  }

  send(method, params = {}) {
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((ok, bad) => this.waiting.set(id, { ok, bad }));
  }

  /// Evaluate in the page and return the value. A thrown exception in
  /// the page is raised here rather than answering undefined — a silent
  /// undefined reads as "the condition is false" and would simply wait.
  async eval(expression) {
    const r = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) {
      throw new Error(r.exceptionDetails.text + ": " + expression.trim().slice(0, 80));
    }
    return r.result.value;
  }

  /// Poll a condition rather than race a fixed sleep: acting on a page
  /// that has not finished rendering is the silent staleness its
  /// callers exist to remove.
  async until(expression, label, ms = 120000) {
    const deadline = Date.now() + ms;
    for (;;) {
      if (await this.eval(`!!(${expression})`)) return;
      if (Date.now() > deadline) throw new Error(`timed out waiting for ${label}`);
      await new Promise((r) => setTimeout(r, 150));
    }
  }
}

/// Where a browser launched with `--remote-debugging-port=0` ended up.
/// The port file appears before it is complete, so the target line is
/// what says it is safe to read.
async function devtoolsUrl(profile) {
  const portFile = path.join(profile, "DevToolsActivePort");
  for (let i = 0; i < 200; i++) {
    if (fs.existsSync(portFile)) {
      const [port, target] = fs.readFileSync(portFile, "utf8").trim().split("\n");
      if (target) return { port, http: `http://127.0.0.1:${port}` };
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error("browser never announced its debugging port");
}

module.exports = { Devtools, devtoolsUrl };
