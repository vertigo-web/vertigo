# Developing vertigo

This file is for agents working **on** vertigo itself. Writing an application **with**
vertigo? Read [crates/vertigo/docs/agents.md](crates/vertigo/docs/agents.md) instead.

## Layout

- `crates/vertigo` - the library
  - `src/reactive` - the reactive graph (`Value`, `Computed`, transactions)
  - `src/reactive_old` - the previous graph, compiled only in tests, for `just reactive-compare`.
    Don't build on it.
  - `src/dom`, `src/dom_macro`, `src/render` - DOM nodes, what `dom!` expands to, list and
    value renderers
  - `src/fetch` - `RequestBuilder`, `LazyCache`, `LazyListCache`
  - `src/driver_module` - the bridge to JavaScript. The browser side is TypeScript in
    `src_js/`, bundled by rollup into `wasm_run.js` and, for what it imports dynamically,
    chunks in `chunks/`, which are committed. A chunk may import nothing (vertigo-cli renames
    `wasm_run.js`), and `rollup.config.mjs` fails the build otherwise. `rollup` is pinned in
    `package.json`: a chunk's name is a hash that changes with rollup's version, so a newer
    rollup is a commit of its own, with the chunks it builds.
  - `docs/` - long-form guides embedded in rustdoc as `vertigo::guides::*` (see `src/lib.rs`)
- `crates/vertigo-macro` - the proc macros: `dom!`, `css!`, `#[component]`, `#[store]`,
  `bind!` and friends
- `crates/vertigo-cli` - the `vertigo` binary (`new`, `build`, `serve`, `watch`), the SSR
  server, mounting into actix-web. Project templates are in `src/new/fe_template` and
  `src/new/fs_template`.
- `crates/vertigo-testing` - the browser harness for end-to-end tests, used by this repo's
  suites and by downstream apps
- `demo/app` with `demo/server` - an app using most features; `examples/` - small apps
- `tests/` - end-to-end suites (`basic`, `demo`), benchmarks, CI shell checks
- `docs/` - `CHANGELOG.md` (symlinked into the crate), `TUTORIAL.md`, release notes

## Commands

`just` lists all recipes. It needs just 1.46 or newer.

- `just ci` - what CI runs: clippy on `wasm32-unknown-unknown` and on the host, all tests,
  `cargo fmt --check`, version and license checks, JS tests, the `vertigo new` template check.
  Run it before calling a change done.
- `just unit-tests` - `cargo test --all-features`
- `just e2e-tests` - the `basic` and `demo` suites in Chrome. The harness starts chromedriver.
  Add `--headed` to watch.
- `just js-build` - rebuild `wasm_run.js` and its chunks after changing anything in `src_js/`.
  Commit them all: a chunk's name changes with its content.
- `just js-test`, `just js-lint`
- `just demo-watch` - the demo on <http://localhost:4444>, rebuilt on change. It never exits.
- `just bench`, `just bench-compare` - benchmark suites. Three of them need a WebDriver on
  `localhost:9515`.

## Rules

- Clippy denies `unwrap_used` and `expect_used` workspace-wide. Lint for wasm too
  (`just clippy-wasm32`): the library code runs there.
- Every user-visible change gets an entry under `## Unreleased` in `docs/CHANGELOG.md`, in
  `### Added`, `### Changed` or `### Fixed`.
- Public items get a doc example that runs as a doctest.
- When a change affects how apps are written, update, where they show it:
  - [crates/vertigo/docs/agents.md](crates/vertigo/docs/agents.md) - its snippets are
    doctests, so a broken one fails `cargo test`
  - `docs/TUTORIAL.md`
  - the project templates in `crates/vertigo-cli/src/new/`
- The major and minor version are also in `VERTIGO_COMPAT_VERSION_*` in
  `src_js/index.ts`. `tests/check_versions.sh` fails when they and the built `wasm_run.js`
  don't match `Cargo.toml`.
- Commit messages: `feat: ...` or `fix: ...`, saying what the change does for the user.
