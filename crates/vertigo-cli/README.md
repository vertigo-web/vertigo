# vertigo-cli - packaging tool for vertigo library

This package provides `vertigo` binary that allows to create and package vertigo-based projects.

1. Runs build
2. Gathers artifacts produced by the build and `vertigo` / `vertigo-macro` libraries:
    - `index.html`
    - `wasm_run.js`
    - `your_project.wasm`
    - _included static files_
3. Optimizes your .wasm file using `wasm-opt`
4. Adds hashes to filenames[^hashes] (to bypass borwsers cache)
5. Places everything in the `build` dictionary

## Installation

```sh
cargo install --force vertigo-cli
```

[^hashes]: Except hashes for included static files - these are computed by vertigo-macro lib.

## Usage

### Generate new project

```sh
vertigo new my_blog
```

### Build the project

```sh
cd my_blog
vertigo build
```

Point your browser to the `./build` directory to check the build.
