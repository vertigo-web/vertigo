Reactive webassembly
===================

Installation and usage
--------------

If you already have rust installed then you have to add webassembly to your toolchain:
- `rustup target add wasm32-unknown-unknown`

Install wasm-pack
- `cargo install wasm-pack`

Install a simple static resource http server
- `cargo install basic-http-server`

Install cargo-watch
- `cargo install cargo-watch`

then build and run project using:
- `./start.sh` or `./start_dev.sh`

eventually terminal will let you know that app is available under http://localhost:3000/

<br />

Basic commands
--------------
- `cargo doc` - Build build documentation
- `cargo doc --open` - Open the documentation in the browser
- `cargo update --dry-run` - Checking for updates
- `rustup target add wasm32-unknown-unknown` - add webassembly

Links
--------------
- `https://rustwasm.github.io/docs/book/reference/crates.html`
- `https://rustwasm.github.io/docs/wasm-bindgen/`
- `https://rustwasm.github.io/book/`
- `https://webassembly.org/`
- `https://github.com/IMI-eRnD-Be/wasm-run`
- `https://rustwasm.github.io/wasm-bindgen/`
- `https://doc.rust-lang.org/cargo/`
- `https://doc.rust-lang.org/book/`
- `https://chinedufn.github.io/percy`


No std
--------------
- `https://rahul-thakoor.github.io/using-no-standard-library-crates-with-webassembly/`
- `https://justjjy.com/Rust-no-std`
- `https://rust-embedded.github.io/book/intro/no-std.html`
- `https://doc.rust-lang.org/core/`
- `https://doc.rust-lang.org/nightly/alloc/index.html`
- `https://crates.io/crates/serde-json-wasm`
- `https://crates.io/crates/serde-wasm-bindgen`
- `https://crates.io/crates/intrusive-collections`
- `https://github.com/rust-embedded/awesome-embedded-rust#no-std-crates`

