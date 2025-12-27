<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## Unreleased

### Added

* vertigo-cli: `--threads` parameter for `serve` command, which allows to specify number of threads to use for processing requests
  (defaults to 2 in `watch` mode, and number of CPU cores in `serve` mode)

### Fixed

* Fixed parsing `animation-` rules in `css!` macro.

### Changed

* vertigo-cli: Migrated to actix-web

## 0.9.1 - 2025-12-18

### Fixed

* vertigo-cli: `--mount-point` option fixed (broken by hydration)
* Nested `Computed` recomputations [#472]
* Fixed missing styles in empty `<head/>`

### Internals

* Optimized SSR metadata handling [#471]
* vertigo-cli: No unwraps [#470]

## 0.9.0 - 2025-12-01

### Added

* `#[store]` macro which wraps a function to be used as a store generator
* `AutoJsJson`: Added `rename_all` container attribute, and `rename` field attribute [#406]
* `AutoJsJson`: Support for `JsJson` type for dynamic schema [#393]
* `SsrFetchCache` - Cache passed in HTML so the browser doesn't need to refetch the data already used during SSR [#413] [#414]
* Hydration (no unnecessary DOM nodes re-creation after page load) [#356]

### Fixed

* Fixed `Invalid “SameSite“ value for cookie` error
* vertigo-cli: Parse but ignore router changes and JS expressions during SSR. [#407]

### Changed

* Reimplemented communication between WASM and JS

### Removed

* `vertigo-suspense` (not very usable anyway)
* `JsValue` (replaced with `JsJson`)

### Internals

* Convenient impls of primitives for JsJson [#418]
* `v-component` and `v-css` was breaking tests in release mode [#396]

## 0.8.3 - 2025-09-29

### Fixed

* vertigo-cli: Workaround for WASM instantiation on different rust versions.

## 0.8.2 - 2025-09-10

### Added

* Implemented `Add` and `AddAssign` for `Css` to easy add multiple css'es to element (`<div css={css1 + css2} />`)
* vertigo-cli: `--wasm-preload` parameter for `serve` command, which makes the browser start loading wasm script earlier.

## 0.8.1 - 2025-08-02

### Fixed

* Restored `ToComputed<Resource<Rc<T>>>` implementation for `LazyCache<T>`.

### Changed

* `AttrGroup` now holds Rc's to callbacks so it now implements `Clone`.

## 0.8.0 - 2025-07-15

### Added

* Tailwind support (internal rust-only, and external node-based) [#353]
* `js!` macro which allows to evaluate simple JavaScript expressions. [#372]
* Added `v-css` and `v-component` attributes in rendered DOM to help debugging (added only in debug mode) [#367]

### Changed

* `Value::set` now doesn't trigger graph update if new value is the same as the old one. [#368]

  This means, `T` should now implement `PartialEq`.
  `Value::set_force` was introduced for `T` which doesn't implement `PartialEq`
  but this method always updates graph just as the old `set` method.

* `on_click` attribute now provides `ClickEvent` to allow preventing default or stopping propagation.

### Fixed

* `DomElement::get_ref()` [#375]

### Removed

* `window!` and `document!` macros (replaced by `js!`).

[#353]: https://github.com/vertigo-web/vertigo/issues/353
[#367]: https://github.com/vertigo-web/vertigo/issues/367
[#368]: https://github.com/vertigo-web/vertigo/issues/368
[#372]: https://github.com/vertigo-web/vertigo/issues/372
[#375]: https://github.com/vertigo-web/vertigo/issues/375

## 0.7.2 - 2025-06-10

### Fixed

* Browser warning about missing source map [#361]

### Changed

* vertigo-cli: `watch` command now logs local time, can be changed using `--log-local-time` parameter [#354]

[#361]: https://github.com/vertigo-web/vertigo/issues/361
[#354]: https://github.com/vertigo-web/vertigo/issues/354

## 0.7.1 - 2025-05-29

### Added

* vertigo-cli: `--release-mode` and `--wasm-opt` parameters for `build` and `watch` commands [#358]
* vertigo-cli: `--watch-ignore-lists` parameter to ignore irrelevant files during watch (defaults to .gitignore) [#351]
* vertigo-cli: `--global-ignores` to add custom wildcards to ignore during watch [#351]

### Fixed

* Visibility in `component` macro [#357]

[#358]: https://github.com/vertigo-web/vertigo/issues/358
[#351]: https://github.com/vertigo-web/vertigo/issues/351

## 0.7.0 - 2025-05-03

### Added

* Dynamic/optional attributes, attributes grouping, attributes spreading [#317]
* vertigo-cli: `--mount-point` parameter for `serve` command, which allow to embed app in f. ex. `example.com/mount/point` endpoint [#346]

### Changed

* Moved from `rsx` to `rstml`, `syn` 1.0 to 2.0 [#331]
* Replaced `OrderedMap` with `BtreeMap` [#322]
* Css classes in single `<style>` element [#328]

### Fixed

* `DomDebugFragment::from_cmds()` fails to debug styles when custom classes used [#335]
* vertigo-cli: Prevent reformatting HTML in `<pre>` during SSR [#342]
* vertigo-cli: Keep original order of CSS rules around media-queries

[#317]: https://github.com/vertigo-web/vertigo/issues/317
[#346]: https://github.com/vertigo-web/vertigo/issues/346
[#331]: https://github.com/vertigo-web/vertigo/issues/331
[#322]: https://github.com/vertigo-web/vertigo/issues/322
[#328]: https://github.com/vertigo-web/vertigo/issues/328
[#335]: https://github.com/vertigo-web/vertigo/issues/335
[#342]: https://github.com/vertigo-web/vertigo/issues/342

## 0.6.4 - 2025-03-26

### Fixed

* vertigo-cli: `watch` now keeps watching even if browser lands directly on non-200 page [#329]
* `DomDebugFragment::to_pseudo_html` now renders all deterministically using BtreeMap so it can be used in unit tests

[#329]: https://github.com/vertigo-web/vertigo/issues/329

## 0.6.3 - 2025-03-01

### Fixed

* Invalid lowercase http methods in requests from inside SSR

## 0.6.2 - 2025-02-27

### Added

* `Driver::set_status` method to allow responding with custom HTTP status code during SSR [#316]
* `css` attribute in `dom!` macro now accepts `&Css` (referenced) for convenience
* `on_submit` in `<form>`

### Changed

* vertigo-cli: Increased statics max-age in Cache-Control header to 1 year to match Google's Lighthouse recommendations
* vertigo-cli: Improved error messages when building and watching

### Fixed

* Intercept inserting multiple html/head/body tags in DOM [#297]
* Removed `unreachable!()` and `unwrap()` from `serve` runtime [#321]
* vertigo-cli: Missing `remove_attr` command in server-side rendering

[#297]: https://github.com/vertigo-web/vertigo/issues/297
[#321]: https://github.com/vertigo-web/vertigo/issues/321

## 0.6.1 - 2024-12-18

### Added

* `Driver::utc_now` (Gets current UTC timestamp)
* `Driver::timezone_offset` (Gets browsers time zone offset in seconds)
* `chrono::NaiveDate` support in `AutoJsJson`
* `LazyCache::<T>::new_resource()` helper
* `ToComputed` impls for primitive types

### Changed

* Hush excessive logging when no Content-Type or cookie provided

### Fixed

* Docstrings and other attributes in `component!` macro

## 0.6.0 - 2024-08-02

### Added

* `Reactive` trait that allows generic components to be more flexible with props
* `BTreeMap` and `chrono::DateTime<Utc>` support in `AutoJsJson`
* `#[js_json(default = "None")]` attribute to `AutoJsJson`
* `JsJson` implementation for unit type `()`
* All http methods in `FetchMethod`
* `history_replace` method in `Driver`
* Minification of `wasm_run.js`
* vertigo-cli: `--add-watch-path` to `watch` command
* vertigo-cli: `--wasm-run-source-map` to `build` and `watch` command

### Fixed

* Missing hash part in history router
* vertigo-cli: Missing `Cache-Control` header for statics

## 0.5.0 - 2024-04-05

### Added

* `window!` and `document!` macro to allow invoking simple JavaScript commands
* `Driver::plains` method to allow responding with plaintext pages
* In `css!` macro there is now possibility to reference a class created by another `css!` using `[]` brackets
* Enums nad newtypes support in `AutoJsJson`
* `bind!` macro now accepts namespaced variables, f. ex. `bind!(state.value, || value + 100)`
* Components now accept value without attribute name if the names matches (`color={color}` → `{color}`)
* In `dom!` macro `..` operator now spreads iterable into children (`<ul>{..items}</ul>`)

### Changed

* Hashing of bundled files shortened from SHA256 to CRC64/Base64 to have file names shorter

### Fixed

* Component embedding using non-local name (f. ex. `<my_module::MyComponent />`)
* Raw field name support in AutoJsJson derive macro
* `component!` macro copying attributes to struct (and doc-strings)
* `css!` macro resolving expressions in `url`
* vertigo-cli: Watch script now attached inside body tag

## 0.4.3 - 2024-02-28

### Fixed

* vertigo-cli: Don't html-escape styles embedded during SSR
* vertigo-cli: Don't panic when missing root html element
* vertigo-cli: Allow missing "head" element
* Removed panics/unwraps from `dom!` macro

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
* In `dom!` macro, name of attribute can be omitted if variable name is the same (`on_click={on_click}` can be shortened to `{on_click}` )
* vertigo-cli: Error message popup can now be dismissed

### Removed

* `DomFragment`

## 0.2.0-alpha - 2023-03-15

### Added

* `vertigo-cli` packaging tool with commands `new`, `build`, `watch` and `serve`
* Server-side rendering built in `vertigo-cli`
* `JsJson` data structure to communicate with JS world without string serialization,
* `AutoJsJson` macro for creating `JsJson` from structures and structures from `JsJson`
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
