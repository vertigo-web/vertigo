# Needs just 1.46 or newer - older versions reject the [parallel] and [arg] attributes, and
# with them the whole file.
set ignore-comments

# List available recipes
default:
    @just --list
    @echo
    @echo 'Recipes marked [OPTIONS] take options, listed by: just --usage <recipe>'

# Unit tests
unit-tests:
    cargo test --all-features

# Compare old vs new reactive graph performance
reactive-compare:
    cargo test --release -p vertigo --lib reactive_old::compare -- --nocapture --test-threads=1

# All four benchmark suites (NOTE: three of them need a WebDriver on localhost:9515)
bench: ssr-bench hydration-bench reactive-bench dom-bench

# All four suites, each against its own target/bench/<suite>/latest.json
bench-compare: ssr-bench-compare hydration-bench-compare reactive-bench-compare dom-bench-compare

# Reactive graph benchmark in a browser (NOTE: WebDriver on localhost:9515 needs to be running)
reactive-bench:
    cargo test --package fantoccini-tests --test reactive_bench -- --ignored --nocapture

# Reactive graph benchmark against an earlier run (defaults to the last)
reactive-bench-compare baseline="target/bench/reactive/latest.json":
    VERTIGO_REACTIVE_BENCH_BASELINE={{ quote(baseline) }} cargo test --package fantoccini-tests --test reactive_bench -- --ignored --nocapture

# Reactive framework end to end - graph, DOM commands, rendering (NOTE: WebDriver on localhost:9515 needs to be running)
dom-bench:
    cargo test --package fantoccini-tests --test dom_bench -- --ignored --nocapture

# End-to-end benchmark against an earlier run (defaults to the last)
dom-bench-compare baseline="target/bench/dom/latest.json":
    VERTIGO_DOM_BENCH_BASELINE={{ quote(baseline) }} cargo test --package fantoccini-tests --test dom_bench -- --ignored --nocapture

# Hydration benchmark in a browser (NOTE: WebDriver on localhost:9515 needs to be running)
hydration-bench:
    cargo test --release --package fantoccini-tests --test hydration_bench -- --ignored --nocapture

# Hydration benchmark against another implementation's run (defaults to the last)
hydration-bench-compare baseline="target/bench/hydration/latest.json":
    VERTIGO_HYDRATION_BENCH_BASELINE={{ quote(baseline) }} cargo test --release --package fantoccini-tests --test hydration_bench -- --ignored --nocapture

# Server-side rendering benchmark, per phase (native - no browser, no WebDriver)
ssr-bench:
    # RUST_LOG=warn: the host logs nothing above warn on a healthy run, so anything this
    # surfaces is a problem worth seeing next to the numbers.
    # --release is load-bearing and unique to this suite: the other three measure wasm
    # inside a browser, this one measures the host.
    RUST_LOG=warn cargo test --release --package fantoccini-tests --test ssr_bench -- --ignored --nocapture

# SSR benchmark against an earlier run (defaults to the last)
ssr-bench-compare baseline="target/bench/ssr/latest.json":
    RUST_LOG=warn VERTIGO_SSR_BENCH_BASELINE={{ quote(baseline) }} cargo test --release --package fantoccini-tests --test ssr_bench -- --ignored --nocapture

# Basic and demo suites in a browser (the harness starts chromedriver; E2E_WEBDRIVER=http://localhost:9515 uses a running one)
[arg('E2E_HEADLESS', long='headed', value='0', help="Show the browser window (E2E_HEADLESS=0)")]
[arg('E2E_RELEASE', long='release', value='1', help="Build the apps in release mode (E2E_RELEASE=1)")]
[arg('E2E_WASM_OPT', long='wasm-opt', value='1', help="Run wasm-opt on the apps, as a shipped build has it (E2E_WASM_OPT=1)")]
[arg('E2E_SKIP_BUILD', long='skip-build', value='1', help="Use the last build (E2E_SKIP_BUILD=1)")]
e2e-tests $E2E_HEADLESS=env('E2E_HEADLESS', '1') $E2E_RELEASE=env('E2E_RELEASE', '') $E2E_WASM_OPT=env('E2E_WASM_OPT', '') $E2E_SKIP_BUILD=env('E2E_SKIP_BUILD', ''):
    # The options are exported as the E2E_* variables the harness reads. An empty one counts
    # as off, so leaving an option out is the same as not setting its variable.
    cargo test --package fantoccini-tests --test basic --test demo -- --ignored

# Click through every demo tab in a browser (the harness starts chromedriver; E2E_WEBDRIVER=http://localhost:9515 uses a running one)
[arg('E2E_HEADLESS', long='headed', value='0', help="Show the browser window (E2E_HEADLESS=0)")]
[arg('E2E_RELEASE', long='release', value='1', help="Build the apps in release mode (E2E_RELEASE=1)")]
[arg('E2E_WASM_OPT', long='wasm-opt', value='1', help="Run wasm-opt on the apps, as a shipped build has it (E2E_WASM_OPT=1)")]
[arg('E2E_SKIP_BUILD', long='skip-build', value='1', help="Use the last build (E2E_SKIP_BUILD=1)")]
demo-tests $E2E_HEADLESS=env('E2E_HEADLESS', '1') $E2E_RELEASE=env('E2E_RELEASE', '') $E2E_WASM_OPT=env('E2E_WASM_OPT', '') $E2E_SKIP_BUILD=env('E2E_SKIP_BUILD', ''):
    cargo test --package fantoccini-tests --test demo -- --ignored

# Ci tests
ci:
    cargo clippy --locked -p vertigo -p vertigo-macro --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings
    cargo clippy --locked -p vertigo-demo -p vertigo-example-counter -p vertigo-example-router -p vertigo-example-trafficlights --all-features --tests --target wasm32-unknown-unknown -- -Dwarnings
    cargo test --locked --all-features
    # The shipping configuration of vertigo-cli. Everything above passes --all-features,
    # which turns `ssr-timings` on - so without this line the build that actually ships is
    # the one nothing lints.
    cargo clippy --locked -p vertigo-cli --tests -- -Dwarnings
    cargo fmt
    tests/check_versions.sh
    tests/js_tests.sh
    tests/check_vertigo_new.sh

# All tests (e2e-tests includes the demo suite)
all-tests: unit-tests e2e-tests

# Demo clean
clean:
    cargo clean

# clean
clean-build:
    rm -rf build build-reactive-bench build-dom-bench build-demo build-ssr-bench build-ssr-demo build-hydration-bench build-hydration-demo

# Run clippy for wasm32 target
clippy-wasm32:
    cargo clippy --all-features --target wasm32-unknown-unknown -p vertigo -p vertigo-macro -p vertigo-demo -p vertigo-example-counter -p vertigo-example-router -p vertigo-example-trafficlights

# Demo tasks - debug mode
demo-debug-api:
    cargo run --bin vertigo-demo-server

# Demo debug watch
demo-debug-watch:
    cargo run --bin vertigo -- watch vertigo-demo --dest-dir=demo_build --wasm-run-source-map --env ws_chat=ws://127.0.0.1:3333/ws --env ws_collection=ws://127.0.0.1:3333/ws-collection

# Demo watch
[parallel]
demo-watch: demo-debug-api demo-debug-watch

# Demo tasks - release mode
demo-serve-api:
    cargo run --bin vertigo-demo-server --release

# Demo serve
demo-serve:
    cargo run --bin vertigo --release -- build vertigo-demo --dest-dir=demo_build --wasm-run-source-map
    cargo run --bin vertigo --release -- serve --dest-dir=demo_build --env ws_chat=ws://127.0.0.1:3333/ws --env ws_collection=ws://127.0.0.1:3333/ws-collection --proxy /api=http://127.0.0.1:3333/api

# Demo
[parallel]
demo: demo-serve-api demo-serve

# Examples tasks
examples-counter:
    cargo run --bin vertigo -- watch vertigo-example-counter --dest-dir=examples/build/counter

# Examples router
examples-router:
    cargo run --bin vertigo -- watch vertigo-example-router --dest-dir=examples/build/router

# Examples trafficlights
examples-trafficlights:
    cargo run --bin vertigo -- watch vertigo-example-trafficlights --dest-dir=examples/build/trafficlights

# JavaScript dev build
js-test:
    npm install
    npm run test

# Js build
js-build:
    npm install
    npx rollup -c

# Lint
lint:
    cargo run --bin lint-project

# Rebase current branch on master, squash to one commit, and force push
git-branch-rebase-and-push: ci
    #!/usr/bin/env bash
    set -eu
    CURRENT_BRANCH=$(git branch --show-current)
    if [ "$CURRENT_BRANCH" = "master" ]; then
        echo "Error: You are on the 'master' branch. This recipe is for feature branches only."
        exit 1
    fi
    git fetch origin master
    git rebase origin/master

    # Find the first commit of this branch since origin/master
    FIRST_COMMIT=$(git rev-list --reverse origin/master..HEAD | head -n 1)
    if [ -z "$FIRST_COMMIT" ]; then
        echo "No commits to squash."
        exit 0
    fi

    # Squash everything into one commit using the first commit's identity
    git reset --soft origin/master
    git commit -C "$FIRST_COMMIT"

    git push origin "$CURRENT_BRANCH" --force-with-lease
