<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## 0.4.3 - Unreleased

### Fixed

* vertigo-cli: Don't html-escape styles embedded during SSR

## 0.4.2 - 2024-02-06

### Fixed

* Lifetimes and generics in `#[component]` macro
* vertigo-cli: Media queries in SSR

## 0.4.1 - 2023-12-02

### Fixed

* Version matching always failed due to `if true` XD

## 0.4.0 - 2023-11-08

### Added

* `LazyCache::forget`
* Check for vertigo/vertigo-cli major.minor versions mismatch. Error is printed on CLI and JavaScript console.

### Fixed

* `LazyCache::force_update` really forces the update even if value not expired
* `JsJson` and `JsValue` list size as u32 - fixes large DOM updates

## 0.3.2 - 2023-07-17

### Added

* `computed_tuple!` macro
* `on_blur`, `on_mouse_down`, `on_mouse_up` event
* `ToComputed` trait

## 0.3.1 - 2023-05-25

### Added

* In `dom!` macro, allow default value for an attribute by passing empty `{}`

### Fixed

* vertigo-cli: Fixed un-captured outputs of commands run during build

## 0.3.0 - 2023-05-01

### Added

* **Breaking:** `dom_element!` macro which returns `DomElement` struct, while `dom!` macro returns `DomNode` now
* Suspense mechanism
* `on_change` event to `<select>`/`<input>`/`<textarea>`
* Env variables passed to application

### Removed

* vertigo-cli: `Cargo` as lib dependency

## 0.2.0 - 2023-03-25

### Added

* `main` macro that wraps a function returning `DomElement` into an app starting entry point

### Changed

* `dom!` macro can now return a list of elements, not only one
* In `dom!` macro, name of attribute can be omitted if variable name is the same (`on_clich={on_click}` can be shortened to `{on_click}` )
* vertigo-cli: Error message popup can now be dismissed

### Removed

* `DomFragment`

## 0.2.0-alpha - 2023-03-15

### Added

* `vertigo-cli` packaging tool with commands `new`, `build`, `watch` and `serve`
* Server-side rendering built in `vertigo-cli`
* `JsJson` data structure to communicate with JS world without string serialization,
* `AutoJsJson` macro for creating `JsJson` from structures and structures from `Jsjson`
* A warning in JS console if developer tried to get a value already set to be changed during transaction
* `@media` queries support in CSS
* `Driver::cookie_set_json` and `Driver::cookie_get_json` for storing `JsJson`-enabled structures in a cookie
* `Driver::history_back()` method invoking `history.back()` on window
* `html_entities` to ease insertion of uncommon letters and symbols in `dom!` macro
* `on_load` event

### Changed

* Renamed DomCommentCreate to DomFragment
* `start_app` doesn't require state

### Removed

* `css_fn!` and `css_fn_push!` macros (not very useful, problems with error reporting in proper place)
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
