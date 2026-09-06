#!/usr/bin/env sh
# Open the microsoft/winget-pkgs pull request for the generated manifests
# (release.yml winget-pr, plan v2.29 step 10, O71). $1 = the version
# just published; WINGET_TOKEN = a token of the account that forks
# winget-pkgs (public_repo). Without the token: a notice and exit 0 —
# the by-hand path is docs/RELEASE.md §3. Everything goes through gh's
# API: the fork is synced, a branch cut from its master, the three
# files put by the contents API, the pull request opened — no clone of
# a multi-gigabyte repository, no credential in any URL.
set -eu
ver=$1
id=skymanbp.CodeEraser
dir="manifests/s/skymanbp/CodeEraser/$ver"
src="packaging/winget/$dir"
[ -d "$src" ] || {
    echo "$src is missing — run node scripts/packaging.js and commit before tagging"; exit 1; }
if [ -z "${WINGET_TOKEN:-}" ]; then
    echo "::notice::WINGET_TOKEN is not set — manifests not submitted; open the microsoft/winget-pkgs pull request by hand from $src (docs/RELEASE.md §3)"
    exit 0
fi
export GH_TOKEN="$WINGET_TOKEN"
me=$(gh api user -q .login)
fork="$me/winget-pkgs"
if ! gh api "repos/$fork" >/dev/null 2>&1; then
    gh repo fork microsoft/winget-pkgs --clone=false
    # a fresh fork is populated asynchronously; poll until the API sees
    # it (this is waiting on GitHub's own job, not masking a race)
    for _ in 1 2 3 4 5 6; do gh api "repos/$fork" >/dev/null 2>&1 && break; sleep 10; done
fi
gh repo sync "$fork" --source microsoft/winget-pkgs --branch master
base=$(gh api "repos/$fork/git/ref/heads/master" -q .object.sha)
branch="codeeraser-$ver"
# re-runnable: an attempt that died after this point left the branch
# behind, and POST /git/refs answers 422 on it — move it instead
if gh api "repos/$fork/git/ref/heads/$branch" >/dev/null 2>&1; then
    gh api -X PATCH "repos/$fork/git/refs/heads/$branch" -f sha="$base" -F force=true >/dev/null
else
    gh api -X POST "repos/$fork/git/refs" -f ref="refs/heads/$branch" -f sha="$base" >/dev/null
fi
for f in "$src"/*.yaml; do
    gh api -X PUT "repos/$fork/contents/$dir/$(basename "$f")" \
        -f message="$id $ver" -f branch="$branch" -f content="$(base64 -w0 "$f")" >/dev/null
done
# first submission or a new version: the repository's own convention
kind="New version"
gh api "repos/microsoft/winget-pkgs/contents/manifests/s/skymanbp/CodeEraser" >/dev/null 2>&1 || kind="New package"
# same re-run story: the pushes above already updated an open pull
# request's branch, so "already exists" is success; anything else refuses
if out=$(gh pr create --repo microsoft/winget-pkgs --head "$me:$branch" \
    --title "$kind: $id version $ver" \
    --body "Generated from the SHA256-pinned release manifest: https://github.com/skymanbp/CodeEraser/blob/v$ver/plugin/bin/manifest.env — the installer URL and hash are the pins the release's tag phase verified before publishing." 2>&1); then
    echo "$out"
else
    case "$out" in
        *"already exists"*) echo "::notice::a pull request for $me:$branch is already open — its branch now carries $ver" ;;
        *) echo "$out"; exit 1 ;;
    esac
fi
