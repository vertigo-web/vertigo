# Notes for coding agents

This is a fullstack [vertigo](https://crates.io/crates/vertigo) app. The frontend is Rust
compiled to WebAssembly. The backend is an actix-web server that serves it, rendering each page
on the server first (SSR), and can serve an API.

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

- `just run-be` - builds the frontend into `build/`, then runs the backend on
  <http://127.0.0.1:8080>. It never exits: run it in the background.
- `just watch-fe` - rebuilds the frontend on every change, serving it on its own on
  <http://127.0.0.1:4444>, without the backend's API. It never exits either.
- `just build-fe` - one frontend build, the quickest check that the frontend compiles
- `just lint` - clippy

## Layout

- `frontend/src/lib.rs` - the app's entry point, the `#[main]` function. Keep it at the end of
  the file.
- `backend/src/main.rs` - the server. Add API routes as actix services there: the vertigo app
  answers every path no other route takes. Relative URLs the frontend fetches during SSR go to
  this server, through `ssr_fetch_base`.
