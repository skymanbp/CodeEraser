// codeeraser.dev is a Cloudflare Pages project with NO git integration
// (docs/RELEASE.md §3): pushing main never deploys site/. This is the
// manual chain in one process so the token hygiene cannot be skipped
// half-way — the master token in .secret (gitignored) has no Pages
// rights of its own; it mints a one-hour account-scoped `Pages Write`
// token, wrangler deploys with that, and the temporary token is deleted
// in a finally block whatever wrangler did. No token value is ever
// written to the console.
//
// Usage: node scripts/deploy_site.js   (exit code = wrangler's)
"use strict";
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const root = path.join(__dirname, "..");
const API = "https://api.cloudflare.com/client/v4";
// Account id is public routing metadata (it appears in every dashboard
// URL), not a credential — the same value docs/RELEASE.md §3 prints.
const ACCOUNT = "ef6ce0a8b2c4ba8529b41aa6fd5b4f45";
const PROJECT = "codeeraser";

async function cf(method, route, bearer, body) {
  const res = await fetch(API + route, {
    method,
    headers: { Authorization: `Bearer ${bearer}`, "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const payload = await res.json();
  // API error bodies name the failing policy, never the request bearer
  if (!res.ok || !payload.success) {
    throw new Error(`${method} ${route} -> HTTP ${res.status}: ${JSON.stringify(payload.errors)}`);
  }
  return payload.result;
}

const stamp = (d) => d.toISOString().replace(/\.\d{3}Z$/, "Z");

async function mint(master) {
  const groups = await cf("GET", "/user/tokens/permission_groups", master);
  const pagesWrite = groups.filter((g) => g.name === "Pages Write");
  if (pagesWrite.length !== 1) {
    throw new Error(`expected exactly one 'Pages Write' group, got ${pagesWrite.length}`);
  }
  const now = Date.now();
  const expires = stamp(new Date(now + 60 * 60 * 1000));
  const made = await cf("POST", "/user/tokens", master, {
    name: `ce-site-deploy-${stamp(new Date(now)).replace(/[-:]/g, "")}`,
    policies: [{
      effect: "allow",
      resources: { [`com.cloudflare.api.account.${ACCOUNT}`]: "*" },
      permission_groups: [{ id: pagesWrite[0].id, name: "Pages Write" }],
    }],
    not_before: stamp(new Date(now - 5 * 60 * 1000)),
    expires_on: expires,
  });
  console.log(`[mint] temp token id=${made.id} expires=${expires}`);
  return made;
}

function deploy(tempValue) {
  const args = ["wrangler", "pages", "deploy", "site", "--project-name", PROJECT, "--commit-dirty=true"];
  console.log(`[deploy] $ npx ${args.join(" ")}`);
  // npx is npx.cmd on Windows, which Node refuses to spawn without a shell
  const run = spawnSync("npx", args, {
    cwd: root,
    shell: process.platform === "win32",
    stdio: "inherit",
    env: { ...process.env, CLOUDFLARE_API_TOKEN: tempValue, CLOUDFLARE_ACCOUNT_ID: ACCOUNT },
  });
  const status = run.status === null ? 1 : run.status;
  console.log(`[deploy] wrangler exit=${status}`);
  return status;
}

async function main() {
  // trim() strips a UTF-8 BOM too: U+FEFF is ECMAScript WhiteSpace
  const master = fs.readFileSync(path.join(root, ".secret"), "utf8").trim();
  const temp = await mint(master);
  let status = 1;
  try {
    status = deploy(temp.value);
  } finally {
    await cf("DELETE", `/user/tokens/${temp.id}`, master);
    console.log(`[cleanup] temp token ${temp.id} deleted`);
  }
  process.exit(status);
}

main().catch((err) => {
  console.error(`[deploy_site] ${err.message}`);
  process.exit(1);
});
