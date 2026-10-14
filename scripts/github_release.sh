#!/usr/bin/env bash

# Tags HEAD and creates a GitHub release from the newest docs/CHANGELOG.md entry.
# Without arguments prints the usage.

set -euo pipefail

CHANGELOG=docs/CHANGELOG.md

usage() {
    cat << EOF
Usage: $0 --dry-run | --release

Tags HEAD with the version of the newest $CHANGELOG entry, pushes the tag and creates
a GitHub release from the entry: tag vX.Y.Z, title "X.Y.Z - YYYY-MM-DD" (the heading),
the entry as the notes.

Run from the main directory of the repository, on master, after the "Prepared X.Y.Z" commit
is pushed.

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

if ! gh auth status > /dev/null 2>&1; then
    fail "gh isn't installed or logged in (gh auth login)"
fi

if [ -n "$(git status --porcelain)" ]; then
    fail "the working tree has uncommitted changes"
fi

git fetch --quiet origin master
if ! git merge-base --is-ancestor HEAD origin/master; then
    fail "HEAD ($(git log -1 --format='%h %s')) isn't on origin/master"
fi

if git rev-parse --quiet --verify "refs/tags/$TAG" > /dev/null; then
    fail "tag $TAG exists already"
fi
if git ls-remote --exit-code --tags origin "refs/tags/$TAG" > /dev/null; then
    fail "tag $TAG exists on origin already"
fi

echo "Tag:    $TAG -> $(git log -1 --format='%h %s')"
echo "Title:  $TITLE"
echo "Notes:"
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

read -r -p "Push tag $TAG and publish the release? [y/N] " ANSWER
if [ "$ANSWER" != "y" ] && [ "$ANSWER" != "Y" ]; then
    echo "Aborted."
    exit 1
fi

git tag "$TAG"
git push origin "$TAG"
gh release create "$TAG" --verify-tag --title "$TITLE" --notes-file - <<< "$NOTES"
