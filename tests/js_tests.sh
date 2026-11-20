#!/bin/bash
set -e

npm install

echo "Linting TS"
npm run lint

echo "Running TS tests"
npm run test

echo "Checking if JS is up to date"
npx rollup -v
npx rollup -c

# A chunk with a new content is a new file, named by it, and the old one is gone. Its name
# changes with the version of rollup, its plugins, typescript, tslib and terser too, which is
# why they are pinned in package.json.
BUNDLE="crates/vertigo/src/driver_module/wasm_run.js crates/vertigo/src/driver_module/chunks"

if ! git diff --no-ext-diff --quiet -- $BUNDLE \
    || [ -n "$(git ls-files --others --exclude-standard -- $BUNDLE)" ]; then
    echo "ERROR: wasm_run.js or its chunks differ"
    git status --short -- $BUNDLE
    exit 1
fi

echo "OK: wasm_run.js and its chunks up to date"

# Size budget for the driver bundle.
#
# Every vertigo app downloads this file before its wasm can do anything, and `vertigo serve`
# only compresses it if compression is on - so the raw number is what a visitor can end up
# paying. Without a gate, growth arrives a few hundred bytes at a time and nobody notices.
#
# The budget is the current size plus a little slack. Raising it is fine when a feature needs
# the room: change the number in the same commit, so it is a decision on the record rather
# than a drift. `tests/bench-report` reports raw and gzip sizes per benchmark run if you want
# the fuller picture.
BUDGET=44000
SIZE=$(wc -c < crates/vertigo/src/driver_module/wasm_run.js)

if [ "$SIZE" -gt "$BUDGET" ]; then
    echo "ERROR: wasm_run.js is ${SIZE} B, over the ${BUDGET} B budget"
    echo "       Shrink it, or raise BUDGET in tests/js_tests.sh and say why."
    exit 1
fi

echo "OK: wasm_run.js is ${SIZE} B, within the ${BUDGET} B budget"

# Each chunk, which an app downloads once it needs what the chunk does. The biggest is the one of
# WebGL and WebGPU together (`gpu.ts`), which share the code sizing canvases. WebGPU's layouts,
# textures from URLs with the mipmaps it makes, multisampled canvases and reads took it from
# 18.8 KB to 24.5 KB (7.4 KB with brotli), which only pages drawing with the GPU download.
CHUNK_BUDGET=25000

for CHUNK in crates/vertigo/src/driver_module/chunks/*.js; do
    SIZE=$(wc -c < "$CHUNK")
    NAME=$(basename "$CHUNK")

    if [ "$SIZE" -gt "$CHUNK_BUDGET" ]; then
        echo "ERROR: ${NAME} is ${SIZE} B, over the ${CHUNK_BUDGET} B budget of a chunk"
        echo "       Shrink it, or raise CHUNK_BUDGET in tests/js_tests.sh and say why."
        exit 1
    fi

    echo "OK: ${NAME} is ${SIZE} B, within the ${CHUNK_BUDGET} B budget of a chunk"
done
