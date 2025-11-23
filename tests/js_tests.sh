#!/bin/bash
set -e

npm install

echo "Running TS tests"
npm run test:hydration

echo "Checking if JS is up to date"
npx rollup -v
npx rollup -c

if ! git diff --no-ext-diff --quiet -- crates/vertigo/src/driver_module/wasm_run.js; then
    echo "ERROR: wasm_run.js differs"
    exit 1
fi

echo "OK: wasm_run.js up to date"
