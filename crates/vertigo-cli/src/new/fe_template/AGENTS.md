# Notes for coding agents

This is a [vertigo](https://crates.io/crates/vertigo) app: Rust compiled to WebAssembly,
rendered on the server first (SSR) and then taken over by the browser.

## Before writing vertigo code

What you remember about vertigo is probably out of date: its API changed a lot between 0.x
versions. Read the guide for the version this project uses. It ships inside the crate, in
`docs/agents.md` next to vertigo's `Cargo.toml`, which this prints:

```sh
cargo metadata --format-version 1 \
  | jq -r '.packages[] | select(.name == "vertigo") | .manifest_path'
```

Without `jq`, look in `~/.cargo/registry/src/*/vertigo-<version>/`, taking the version from
`Cargo.lock`.

Read the guide before your first change. Before using a vertigo API you haven't seen in this
project, grep the crate's `src/` for it.

## Commands

- `vertigo watch` - builds, serves on <http://127.0.0.1:4444> and rebuilds on every change. It
  never exits: run it in the background.
- `vertigo build` - one build, the quickest check that the app compiles for the browser
- `cargo clippy --target wasm32-unknown-unknown` - lint

## Layout

- `src/lib.rs` - the app's entry point, the `#[main]` function. Keep it at the end of the file.
