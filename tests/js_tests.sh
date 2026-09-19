#!/bin/bash
set -e

npm install

echo "Running TS tests"
npm run test

echo "Checking if JS is up to date"
npx rollup -v
npx rollup -c

if ! git diff --no-ext-diff --quiet -- crates/vertigo/src/driver_module/wasm_run.js; then
    echo "ERROR: wasm_run.js differs"
    exit 1
fi

echo "OK: wasm_run.js up to date"

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
BUDGET=31500
SIZE=$(wc -c < crates/vertigo/src/driver_module/wasm_run.js)

if [ "$SIZE" -gt "$BUDGET" ]; then
    echo "ERROR: wasm_run.js is ${SIZE} B, over the ${BUDGET} B budget"
    echo "       Shrink it, or raise BUDGET in tests/js_tests.sh and say why."
    exit 1
fi

echo "OK: wasm_run.js is ${SIZE} B, within the ${BUDGET} B budget"
