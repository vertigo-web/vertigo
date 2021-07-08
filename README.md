Vertigo - reactive webassembly
===================

Installation and usage of demo application
--------------

Make sure you're using nigthly version of rust:
- `rustup default nightly`

Install cargo-make that takes care of all other dependencies:
- `cargo install cargo-make`

Build and run project using:
- `cargo make demo-start`

Eventually terminal will let you know that app is available under http://localhost:3000/

If you want to play around with the code, you can make cargo to watch for your changes:
- `cargo make demo-watch`

Keep in mind that you still need to refresh page in the browser after project recompiles.

Different build profiles
--------------
- `cargo make demo-watch --profile profiling`
- `cargo make demo-watch --profile release`

Some random dev stuff below
===================

Helpful commands
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

