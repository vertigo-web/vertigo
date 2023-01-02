<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->
<!-- markdownlint-disable-next-line first-line-h1 -->
## Unreleased

### Added

* `vertigo-cli` packaging tool with commands `new`, `build` and `serve`
* Server-side rendering built in `vertigo-cli`
* `JsJson` data structure to communicate with JS world without string serialization,
* `AutoJsJson` macro for creating `JsJson` from structures and structures from `Jsjson`
* A warning in JS console if developer tried to get a value already set to be changed during transaction
* `@media` queries support in CSS
* `Driver::cookie_set_json` and `Driver::cookie_get_json` for storing `JsJson`-enabled structures in a cookie
* `Driver::history_back()` method invoking `history.back()` on window
* `html_entities` to ease insertion of uncommon letters and symbols in `dom!` macro

### Removed

* `css_fn!` and `css_fn_push!` macro (not very useful, problems with error reporting in proper place)
* `serde` dependency

## 0.1.1 - 2022-11-10

### Added

* `bind!`, `bind_rc!`, `bind_spawn!` macros
* `driver.get_random()`
* `impl From<Value> for Computed`

### Removed

* BREAKING: Removed `bind`, `bind2`, ... functions

## 0.1.0 - 2022-10-20

### Changed

* Refactored websocket mechanism (internal)

## 0.1.0-beta.5 - 2022-10-18

### Added

* Components with "props"!
* `DomElement::from_parts` for unit-testing purposes

### Changed

* Improved refresh algorithm (internal)
* Simplified context system (internal)
* Refactored callbacks mechanism (internal)

### Removed

* `RefCell`

## 0.1.0-beta.4 - 2022-10-02

### Added

* `hook_keydown` and `on_dropfile` events
* `bind`, `bind2`... functions for creating event handlers

### Changed

* `Driver` object is now global, so there's no need to pass it as parameter in all functions
* `get_value` and `set_value` methods are now `get` and `set`
* `start_app` initialization function now takes an `FnOnce` instead of ready `VDomComponent`
* Dropped `PartialEq` constraint from `Value`, `Computed` and other implementations
* Refactored subsystem for exchanging values between rust and js

### Removed

* Removed virtual dom intermediate in favour of real dom operations
* `vertigo-browserdriver` package - it is now integrated into `vertigo` as the default and only driver
* Callback from `HashRouter` - it can be now treated similarly to `Value`
* `EqBox`

## 0.1.0-beta.3 - 2022-01-22

### Added

* Cookies support in JS Driver

### Changed

* Improved initiation of spawn executor
* Improvements in Graph
* RC-structures, BoxRefCell removed

### Removed

* Removed wasm-bindgen

## 0.1.0-beta.2 - 2021-12-21

### Added

* `start_browser_app` function with optional wasm-logger configuration
* Examples directory
* More docstrings
* Demo: Speed setting in game of life

### Changed

* `wasm-bindgen` is now re-exported in `vertigo-browserdriver` so it is easier to use its proper version

### Removed

* We-alloc usage as it caused memory problems on wasm-js border

## 0.1.0-beta.1 - 2021-12-10

### Added

* Re-exports for AutoMap, Computed, Value, DropResource
* Tutorial
* Some docstrings with examples

### Changed

* LazyCache::force_update - Added parameter with_loading
* Simplified computed refresh function

## 0.1.0-alpha.3 - 2021-11-29

### Added

* Installation and usage notes
* JS Driver - Replacement for `web-sys`
* Instant - Replacement for `std::time::Instant` in browser driver
* LazyCache - A wrapper on Value that make it cache for defined amount of time and is lazily loaded using provided loader function
* SerdeRequest derive macro - allows a structure to be automatically loaded from response or passed in body request using serde library

### Changed

* Reorganized project structure
* Simplified application start
* Performance improvements in browser driver

### Removed

* `wasm-run` and `web-sys` dependency

## 0.1.0-alpha.2 - 2021-05-28

### Added

* FetchBuilder - Allows to configure request before sending
* CSS pseudoselectors support
* Support for MouseEnter, MouseLeave, KeyDown events

### Fixed

* SVG rendering

## 0.1.0-alpha.1 - 2021-01-07

### Added

* HTML/CSS macros - Allows to construct Virtual DOM nodes using HTML and CSS
* Fetch - Allows to fetch data from the internet
* HashRouter - Allows to hook on changes in hash location in url
* Demo: Game of Life - presents possibility of making changes in app state in one transaction

### Fixed

* Leaking subscriptions

## 0.1.0-alpha.0 - 2020-12-23

Initial release

* Virtual DOM - Lightweight representation of JavaScript DOM that can be used to optimally update real DOM
* Reactive dependencies - A graph of values and clients that can automatically compute what to refresh after one value change
* Browser driver - Methods for interacting with real DOM and the browser itself
* AutoMap - HashMap that automatically creates value using passed constructor
* Demo app - example app that tries to use every feature of vertigo
