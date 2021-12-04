Vertigo - reactive webassembly
===================

Features
--------------

* **Virtual DOM** - Lightweight representation of JavaScript DOM that can be used to optimally update real DOM
* **Reactive dependencies** - A graph of values and clients that can automatically compute what to refresh after one value change
* **HTML/CSS macros** - Allows to construct Virtual DOM nodes using HTML and CSS

See [Changelog](/CHANGES.md) for recent features.

Go to **[TUTORIAL](/tutorial.md)** if you want to try.

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

A community, soon to grow
--------------

- Discord server: https://discord.gg/HAXtTeFrAf
