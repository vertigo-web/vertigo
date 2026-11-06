#!/usr/bin/env bash

# Tags HEAD and creates a GitHub release from the newest changelog entry.
# Without arguments prints the usage.

set -euo pipefail

# The changelog is the first of these found
CHANGELOGS=(docs/CHANGELOG.md CHANGELOG.md CHANGES.md)

usage() {
    cat << EOF
Usage: $0 --dry-run | --release

Tags HEAD with the version of the newest changelog entry, pushes the tag and creates
a GitHub release from the entry: tag vX.Y.Z, title "X.Y.Z - YYYY-MM-DD" (the heading),
the entry as the notes. The changelog is the first found of: ${CHANGELOGS[*]}.

Run from the main directory of the repository, on master, after the "Prepared X.Y.Z" commit
is pushed. A run stopped after pushing the tag can be run again: a tag on HEAD is kept.

  --dry-run  print the tag, title and notes, and the checks that would fail, change nothing
  --release  tag, push the tag and create the release (asks first)
EOF
}

DRY_RUN=
case "${1:-}" in
    --dry-run) DRY_RUN=1 ;;
    --release) ;;
    -h | --help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac
if [ $# -gt 1 ]; then
    usage >&2
    exit 2
fi

FAILED=
fail() {
    echo "Error: $1" >&2
    if [ -z "$DRY_RUN" ]; then
        exit 1
    fi
    FAILED=1
}

CHANGELOG=
for CANDIDATE in "${CHANGELOGS[@]}"; do
    if [ -f "$CANDIDATE" ]; then
        CHANGELOG=$CANDIDATE
        break
    fi
done
if [ -z "$CHANGELOG" ]; then
    echo "Error: no changelog here (${CHANGELOGS[*]}), run from the main directory of the repository" >&2
    exit 1
fi

# The first "## " heading and the lines up to the next one, without blank lines around
TITLE=$(grep -m1 '^## ' "$CHANGELOG" | sed 's/^## //')
NOTES=$(awk '
    /^## / { if (seen++) exit; next }
    seen { lines[++n] = $0 }
    END {
        first = 1; while (first <= n && lines[first] == "") first++
        last = n; while (last >= first && lines[last] == "") last--
        for (i = first; i <= last; i++) print lines[i]
    }
' "$CHANGELOG")

if [[ ! $TITLE =~ ^([0-9]+\.[0-9]+\.[0-9]+)\ -\ [0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "Error: the newest $CHANGELOG entry \"$TITLE\" isn't \"X.Y.Z - YYYY-MM-DD\"" >&2
    exit 1
fi
VERSION=${BASH_REMATCH[1]}
TAG="v$VERSION"

if [ -z "$NOTES" ]; then
    fail "the $CHANGELOG entry for $VERSION is empty"
fi

CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
if [ "$CARGO_VERSION" != "$VERSION" ]; then
    fail "Cargo.toml has version $CARGO_VERSION, $CHANGELOG $VERSION"
fi

# Not `gh auth status`: it passes with a revoked or expired token
if [ "$(gh api 'repos/{owner}/{repo}' --jq .permissions.push 2> /dev/null)" != "true" ]; then
    fail "gh can't push to the repository: not installed, not logged in, bad credentials or no access (gh auth login)"
fi

if [ -n "$(git status --porcelain)" ]; then
    fail "the working tree has uncommitted changes"
fi

git fetch --quiet origin master
if ! git merge-base --is-ancestor HEAD origin/master; then
    fail "HEAD ($(git log -1 --format='%h %s')) isn't on origin/master"
fi

# A tag on HEAD is left from a run that stopped before the release, one elsewhere stops it
HEAD_COMMIT=$(git rev-parse HEAD)
LOCAL_TAG=$(git rev-parse --quiet --verify "refs/tags/$TAG^{commit}" || true)
if [ -n "$LOCAL_TAG" ] && [ "$LOCAL_TAG" != "$HEAD_COMMIT" ]; then
    fail "tag $TAG exists already, on $(git log -1 --format='%h %s' "$LOCAL_TAG")"
fi
# An annotated tag comes with a `^{}` line giving the commit it points to
REMOTE_TAG=$(git ls-remote --tags origin "refs/tags/$TAG" "refs/tags/$TAG^{}" | awk -v ref="refs/tags/$TAG" '
    $2 == ref { tag = $1 }
    $2 == ref "^{}" { peeled = $1 }
    END { print peeled ? peeled : tag }
')
if [ -n "$REMOTE_TAG" ] && [ "$REMOTE_TAG" != "$HEAD_COMMIT" ]; then
    fail "tag $TAG exists on origin already, on ${REMOTE_TAG:0:8}"
fi

if gh release view "$TAG" > /dev/null 2>&1; then
    fail "release $TAG exists already"
fi

if [ "$REMOTE_TAG" = "$HEAD_COMMIT" ]; then
    TAG_STATE=" (pushed already)"
elif [ "$LOCAL_TAG" = "$HEAD_COMMIT" ]; then
    TAG_STATE=" (tagged already, not pushed)"
else
    TAG_STATE=
fi
echo "Tag:    $TAG -> $(git log -1 --format='%h %s')$TAG_STATE"
echo "Title:  $TITLE"
echo "Notes:  from $CHANGELOG"
echo
echo "$NOTES"
echo

if [ -n "$DRY_RUN" ]; then
    if [ -n "$FAILED" ]; then
        echo "Dry run: the checks above would stop the release."
        exit 1
    fi
    echo "Dry run: nothing changed."
    exit 0
fi

if [ -n "$REMOTE_TAG" ]; then
    QUESTION="Publish the release?"
else
    QUESTION="Push tag $TAG and publish the release?"
fi
read -r -p "$QUESTION [y/N] " ANSWER
if [ "$ANSWER" != "y" ] && [ "$ANSWER" != "Y" ]; then
    echo "Aborted."
    exit 1
fi

# Only the tag that gets pushed is made locally: a lightweight copy of an annotated tag
# on origin would clash with it on the next fetch
if [ -z "$REMOTE_TAG" ]; then
    if [ -z "$LOCAL_TAG" ]; then
        git tag "$TAG"
    fi
    git push origin "$TAG"
fi
gh release create "$TAG" --verify-tag --title "$TITLE" --notes-file - <<< "$NOTES"
