#!/usr/bin/env sh
# Put the generated Homebrew formula into the tap (release.yml
# homebrew-tap, plan v2.29 step 10, O71). $1 = the version just
# published; HOMEBREW_TAP_TOKEN = a token that can write the tap
# repository. Without the token: a notice and exit 0 — the by-hand
# path is docs/RELEASE.md §3. One contents-API call through gh, so no
# clone and no credential ever touches a URL or the log.
set -eu
ver=$1
formula=packaging/homebrew/Formula/codeeraser.rb
# the REPOSITORY (what the API addresses) and the TAP NAME (what a user
# types) differ: brew strips the `homebrew-` prefix, so the three-part
# `skymanbp/homebrew-codeeraser/...` would look for homebrew-homebrew-codeeraser
tap=skymanbp/homebrew-codeeraser
brewtap=skymanbp/codeeraser
grep -q "/download/v$ver/" "$formula" || {
    echo "$formula does not pin v$ver — run node scripts/packaging.js and commit before tagging"; exit 1; }
if [ -z "${HOMEBREW_TAP_TOKEN:-}" ]; then
    echo "::notice::HOMEBREW_TAP_TOKEN is not set — formula not pushed; copy $formula into the $tap repository by hand (docs/RELEASE.md §3)"
    exit 0
fi
export GH_TOKEN="$HOMEBREW_TAP_TOKEN"
gh api "repos/$tap" >/dev/null 2>&1 || {
    echo "the tap repository $tap does not exist (or the token cannot see it) — create it empty, then re-run this job"; exit 1; }
# the file's current blob sha, when it exists: the API requires it to update
sha=$(gh api "repos/$tap/contents/Formula/codeeraser.rb" -q .sha 2>/dev/null || true)
content=$(base64 -w0 "$formula")
if [ -n "$sha" ]; then
    gh api -X PUT "repos/$tap/contents/Formula/codeeraser.rb" \
        -f message="codeeraser $ver" -f content="$content" -f sha="$sha" >/dev/null
else
    gh api -X PUT "repos/$tap/contents/Formula/codeeraser.rb" \
        -f message="codeeraser $ver" -f content="$content" >/dev/null
fi
echo "tap $tap now carries codeeraser $ver — brew install $brewtap/codeeraser"
