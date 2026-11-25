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
#
# The chunks lie next to `wasm_run.js`, named `name.HASH.js`, with their source maps.
DRIVER="crates/vertigo/src/driver_module"
BUNDLE=("$DRIVER/wasm_run.js" ":(glob)$DRIVER/*.*.js" ":(glob)$DRIVER/*.*.js.map")

if ! git diff --no-ext-diff --quiet -- "${BUNDLE[@]}" \
    || [ -n "$(git ls-files --others --exclude-standard -- "${BUNDLE[@]}")" ]; then
    echo "ERROR: wasm_run.js or its chunks differ"
    git status --short -- "${BUNDLE[@]}"
    exit 1
fi

echo "OK: wasm_run.js and its chunks up to date"

# Size budget for the bootstrap.
#
# `wasm_run.js` is all of vertigo a page downloads and runs before it has loaded (see
# `src_js/boot.ts`), next to the images and styles it shows first. What wasm talks to the browser
# through goes to the runtime, below, so this one has no reason to grow.
BUDGET=1600
SIZE=$(wc -c < crates/vertigo/src/driver_module/wasm_run.js)

if [ "$SIZE" -gt "$BUDGET" ]; then
    echo "ERROR: wasm_run.js is ${SIZE} B, over the ${BUDGET} B budget"
    echo "       Shrink it, or raise BUDGET in tests/js_tests.sh and say why."
    exit 1
fi

echo "OK: wasm_run.js is ${SIZE} B, within the ${BUDGET} B budget"

# Size budget for the runtime.
#
# Every vertigo app downloads this chunk together with its wasm, once the page has loaded, and
# the wasm can do nothing before it is in. `vertigo serve` only compresses it if compression is
# on - so the raw number is what a visitor can end up paying. Without a gate, growth arrives a
# few hundred bytes at a time and nobody notices.
#
# The budget is the current size plus a little slack. Raising it is fine when a feature needs
# the room: change the number in the same commit, so it is a decision on the record rather
# than a drift. `tests/bench-report` reports raw and gzip sizes per benchmark run if you want
# the fuller picture. `localStorage` (getting, keeping and forgetting a value) took it from
# 43.9 KB to 44.4 KB, and giving the focus back (`Driver::remember_focus`) to 45.1 KB, all of it
# in `wasm_run.js` then. Without the bootstrap it is 44.1 KB.
RUNTIME_BUDGET=44300

# Each other chunk, which an app downloads once it needs what the chunk does. The biggest is the
# one of WebGL and WebGPU together (`gpu.ts`), which share the code sizing canvases. WebGPU's
# layouts, textures from URLs with the mipmaps it makes, multisampled canvases and reads took it
# from 18.8 KB to 24.5 KB (7.4 KB with brotli), which only pages drawing with the GPU download.
CHUNK_BUDGET=25000

RUNTIMES=0

for CHUNK in "$DRIVER"/*.*.js; do
    SIZE=$(wc -c < "$CHUNK")
    NAME=$(basename "$CHUNK")

    case "$NAME" in
        runtime.*)
            RUNTIMES=$((RUNTIMES + 1))
            LIMIT=$RUNTIME_BUDGET
            WHICH="RUNTIME_BUDGET"
            OF="the runtime"
            ;;
        *)
            LIMIT=$CHUNK_BUDGET
            WHICH="CHUNK_BUDGET"
            OF="a chunk"
            ;;
    esac

    if [ "$SIZE" -gt "$LIMIT" ]; then
        echo "ERROR: ${NAME} is ${SIZE} B, over the ${LIMIT} B budget of ${OF}"
        echo "       Shrink it, or raise ${WHICH} in tests/js_tests.sh and say why."
        exit 1
    fi

    echo "OK: ${NAME} is ${SIZE} B, within the ${LIMIT} B budget of ${OF}"
done

# The bootstrap imports it by its name, and its budget is found by it too.
if [ "$RUNTIMES" -ne 1 ]; then
    echo "ERROR: ${RUNTIMES} chunks named runtime.*.js, the bootstrap imports one"
    exit 1
fi
