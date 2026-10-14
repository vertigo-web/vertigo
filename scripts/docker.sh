#!/usr/bin/env bash

# Builds the vertigo-cli image and pushes it to Docker Hub.
# Without arguments prints the usage.

set -euo pipefail

IMAGE=${IMAGE:-sfisol/vertigo}
PLATFORMS=${PLATFORMS:-linux/amd64,linux/arm64}

usage() {
    cat << EOF
Usage: $0 --dry-run | --build | --push [--no-latest]

Builds the vertigo-cli image for $PLATFORMS and tags it X.Y.Z, X.Y and latest,
with the version from Cargo.toml.

Run from the main directory of the repository, at the release commit.

  --dry-run    print the build command (of --push, unless --build is given), change nothing
  --build      only build (the result stays in the build cache)
  --push       build and push (asks first)
  --no-latest  leave latest where it is (f. ex. for a fix of an older version)

Environment:
  IMAGE      image name (default: sfisol/vertigo)
  PLATFORMS  platforms to build for (default: linux/amd64,linux/arm64)
EOF
}

DRY_RUN=
MODE=
LATEST=1
for ARG in "$@"; do
    case "$ARG" in
        --dry-run) DRY_RUN=1 ;;
        --build) MODE=build ;;
        --push) MODE=push ;;
        --no-latest) LATEST= ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            usage >&2
            exit 2
            ;;
    esac
done
if [ -z "$MODE" ]; then
    if [ -z "$DRY_RUN" ]; then
        usage >&2
        exit 2
    fi
    MODE=push
fi

VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
if [[ ! $VERSION =~ ^([0-9]+\.[0-9]+)\.[0-9]+$ ]]; then
    echo "Error: Cargo.toml has version \"$VERSION\", not X.Y.Z" >&2
    exit 1
fi
MINOR=${BASH_REMATCH[1]}

TAGS=("$IMAGE:$VERSION" "$IMAGE:$MINOR")
if [ -n "$LATEST" ]; then
    TAGS+=("$IMAGE:latest")
fi

CMD=(docker build --platform "$PLATFORMS")
for TAG in "${TAGS[@]}"; do
    CMD+=(-t "$TAG")
done
if [ "$MODE" = push ]; then
    CMD+=(--push)
fi
CMD+=(.)

echo "${CMD[*]}"

# The build context is the working tree, not the commit
if [ "$MODE" = push ] && [ -n "$(git status --porcelain)" ]; then
    echo "Error: the working tree has uncommitted changes, they would go into the image" >&2
    exit 1
fi

if [ -n "$DRY_RUN" ]; then
    echo "Dry run: nothing changed."
    exit 0
fi

if [ "$MODE" = push ]; then
    read -r -p "Build and push ${TAGS[*]}? [y/N] " ANSWER
    if [ "$ANSWER" != "y" ] && [ "$ANSWER" != "Y" ]; then
        echo "Aborted."
        exit 1
    fi
fi

"${CMD[@]}"
