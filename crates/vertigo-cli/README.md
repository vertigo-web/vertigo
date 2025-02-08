# vertigo-cli

A packaging tool and server for vertigo library

[![crates.io](https://img.shields.io/crates/v/vertigo-cli)](https://crates.io/crates/vertigo-cli)
[![Documentation](https://docs.rs/vertigo-cli/badge.svg)](https://docs.rs/vertigo-cli)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/vertigo-cli.svg)
[![Dependency Status](https://deps.rs/crate/vertigo-cli/0.3.1/status.svg)](https://deps.rs/crate/vertigo-cli/0.3.1s)
[![downloads](https://img.shields.io/crates/d/vertigo-cli.svg)](https://crates.io/crates/vertigo-cli)

This package provides `vertigo` binary that allows to _create_, _build_, _serve_ and _watch_ vertigo-based projects.

Packaging steps performed during _build_ command:

1. Runs cargo build
2. Gathers artifacts produced during the build and by `vertigo` / `vertigo-macro` libraries:
    - `index.html`
    - `wasm_run.js`
    - `your_project.wasm`
    - _included static files_
3. Optimizes your .wasm file using `wasm-opt`
4. Adds hashes to filenames[^hashes] (to bypass browser's cache)
5. Places everything in the `build` dictionary

## Installation

```sh
cargo install --force vertigo-cli
```

## Example usage

### Generate new project

```sh
vertigo new my_blog
```

### Build the project

```sh
cd my_blog
vertigo build
```

### Serve project

```sh
vertigo serve --host 0.0.0.0 --port 8000
```

### Watch project

```sh
vertigo watch --disable-wasm-opt
```

[^hashes]: Except hashes for included static files - these are computed by vertigo-macro library

### Error codes returned from `vertigo-cli` commands

`1` Cant Open Workspace

`2` Cant Parse Workspace

`3` Cant Find Cdylib Member

`4` Package Name Not Found

`5` Build Failed

`6` Build Prerequisites Failed

`7` Watcher Error

`8` Cant Add Watch Dir

`9` Other Process Already Running

`10` Cant Read Wasm Run From Statics

`11` Cant Read Wasm Run Sourcemap From Statics

`12` Cant Write To File

`13` Cant Spawn Child Process

`14` Couldnt Wait For Child Process

`15` Watch Serve Failed

`16` Watch Pipe Broken

`17` New Project Dir Not Empty

`18` New Project Cant Create Dir

`19` New Project Cant Unpack Stub

`20` New Project Can Create Cargo Toml

`21` New Project Can Write To Cargo Toml

`22` Serve Cant Find Http Base Path

`23` Serve Cant Read Index File

`24` Serve Cant Open Port

`25` Serve Wasm Read Failed

`26` Serve Wasm Compile Failed

`27` Serve Path To Url Translation Failed
