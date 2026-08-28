<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## 0.13.0 - unreleased

The reactive graph was rewritten and keyed list rendering was rebuilt around per-key
`Computed`s. See the [reactive graph guide][reactive-graph] and the
[collection guide][collections].

### Added

* A new reactive graph in the `vertigo::reactive` module: transactional, with an equality
  cutoff. See the [guide][reactive-graph]
* `reactive::Graph` - an isolated graph instance, mostly useful in tests
* `reactive::invariants` - the rules the graph enforces, and where you may write
* `reactive::transaction` and `reactive::on_after_transaction`
* `GraphId` - identity of a `Value` or `Computed`
* `RenderValue` - trait form of `render_value` / `render_value_option`, for generic code
* `keyed_computed_list` and `KeyedListItem` - per-key `Computed`s from a reactive list,
  reusing the same `Computed` for a given key across updates (Solid `<For>`-style)
* `MarkerContent` - lets a marker comment report the nodes it keeps in front of itself

### Changed

* **Breaking**: `Computed<T>` requires `T: PartialEq` everywhere, not only for `subscribe`
* **Breaking**: `Value::set` from a compute closure, a `subscribe` callback, or a
  `DropResource` destructor is refused. It used to panic with *"You cannot change the
  source value while the dependency graph is being refreshed"*; the write is now ignored
  and reported through `log::error!`, so one misplaced write no longer takes the
  application down. This follows the call stack, so it covers a `Drop` that runs inside
  such a callback. Writing from event handlers, timers, fetch, `on_after_transaction` and
  `when_connect` (`create`) is unaffected - see the [invariants][invariants]
* **Breaking**: `when_connect` and `Value::with_connect` run their closure after the wave
  that made the node watched (and after `on_after_transaction`), and drop the resource
  after the wave that unwatched it, instead of in the middle of a refresh. `create` may
  write; the destructor must only tear down the external handler
* **Breaking**: a `Computed` recomputes when its dependencies change, not on the next read
* **Breaking**: reading through `transaction(|ctx| ...)` serves the cached value
* **Breaking**: `EmbedDom` is no longer implemented for every owned `T: ToString`. Owned
  support is an explicit list; references stay blanket, so `&MyType` still embeds. For an
  owned value: `impl EmbedDom for MyType { fn embed(self) -> DomNode { self.to_string().embed() } }`
* **Breaking**: `render_list` takes a `Vec<T>` source, its render closure receives
  `&Computed<T>` instead of `&T`, and the key type must implement `Debug`
* **Breaking**: the render closures of `render_list_memo` and `render_resource_list_memo`
  receive `&Computed<T::Value>`; `Loading` and `Error` render as an empty list
* **Breaking**: the mount closure of `DomComment::new_marker` takes a third argument,
  `&MarkerContent`. A marker moved within the same parent no longer re-runs its mount
* Every `render_list` row is preceded by an anchor comment node
* A reactive value embedded in `dom!` - `<span>{my_value}</span>` - now updates its text
  node in place instead of replacing it - 3x faster. Note the DOM
  shape change if you assert on rendered HTML - `<li>{value}</li>` no longer emits a
  trailing `<!-- v -->` inside the element
* `DomText::new_computed` creates its node with the value already set rather than empty,
  and gained a `new_computed_display` sibling for `T: ToString` (`u32`, `bool`, your own
  `Display` types) alongside the existing `T: Into<String>` form
* **Breaking** (`dev`): DOM commands travel to the browser in a flat binary format instead of
  as `JsJson` objects, so `CommandForBrowser::DomBulkUpdate` now carries `commands: Vec<u8>`
  rather than `list: Vec<DriverDomCommand>`. Encode and decode with
  `dev::command_wire::{encode_dom_commands, decode_dom_commands}`. Every command used to cost
  two `BTreeMap`s and three heap `String`s to build, serialize and drop, and repeated its
  field names on the wire once per command; element and attribute names are now interned per
  batch. Nothing above the wire changed - the same commands are emitted in the same order
* `AutoJsJson` encodes a `Vec<u8>` field of an enum variant as `JsJson::Vec` - one
  length-prefixed byte run - which is what it already did for a struct field of that type.
  It previously produced a list with a `JsJson::Number` per byte
* `keyed_computed_list` builds one index per update instead of three, and stamps a single
  key cache instead of rebuilding two.
* Vertigo's own hash maps - the keyed-list index, the list reconciler's middle cache, and
  every `dev::HashMapMut` including the one consulted for each DOM insertion - use an FxHash
  hasher instead of the standard `SipHash-1-3`. This is **not** collision-resistant against
  chosen keys: the worst case is a render that degrades to quadratic on a list keyed by
  attacker-chosen strings, which matters server-side under SSR and not in the browser. It is
  worth about a fifth of the keyed-list improvement above. `dev::HashMapMut` gains a
  defaulted hasher parameter, so a map that wants `RandomState` can still say so
* **Breaking**: `AttrValue` has a new `Static(&'static str)` variant, and `AttrValue::get`
  returns the new `AttrText` instead of `Rc<String>`.
* An element only builds a class merger when it has two sources to merge. It used to build
  one always: 160 bytes behind an `Rc` on every element
* The merger itself is smaller and quieter, the `css` half is boxed so only elements using
  css carry it, and the resolved css class name is cached rather than re-derived whenever
  the class attribute moves
* Only `class` goes through that merger now. Every other attribute is recognised when the
  element is built rather than on every write, and talks to the driver directly, so a
  reactive `href` no longer clones an `Rc` into its subscription
* New `DomElement::attr_static`, which the `dom!` macro emits for an attribute whose name and
  value are both literals - `class="col-md-1"`, `aria-hidden="true"`.
* An application that uses no `css!` no longer links the css engine either. Together with
  the entry above, wasm going from 284 kB to 261 (84 kB to 76 gzipped) and 16% off first
  paint. **An application that does use `css!` keeps the engine and sees no size change**.
* `vertigo new` now writes a `[profile.release]` into the project it generates.
  The fullstack template additionally splits `opt-level` per target through `.cargo/config.toml`,
  so the frontend is built for size and the backend for speed
* The SSR fetch cache is decoded on first use rather than at startup. An application with no
  `LazyCache` never mentions it, so the linker drops the whole decode path producing smaller
  wasm
* Guide `guides::value_synchronize_and_collections` replaced by [`collections`][collections]

### Removed

* **Breaking**: `Dependencies`. Use `transaction`, `Driver::transaction` and
  `Driver::on_after_transaction`
* **Breaking**: `Computed::subscribe_all` - unchanged values no longer notify anybody, so
  it has nothing to report. Use `subscribe`
* **Breaking**: `ValueSynchronize`, `Value::synchronize`, `LazyCache::synchronize` and
  `CacheValue::synchronize`. Use `keyed_computed_list`
* **Breaking**: `Collection` and `CollectionModel`, superseded by `keyed_computed_list`.
  `CollectionKey` stays

### Fixed

* `render_list` corrupted sibling order when reordering or inserting rows whose root is a
  plain element rather than a `render_value` marker
* Moving a row repositions its DOM instead of rebuilding it, so input values, listeners and
  children survive a reorder
* Updating one row of a keyed list no longer costs work proportional to the *square* of the
  list length
* Reordering a keyed list moves only the rows whose order actually changed. The keyed phase
  used to re-insert every row it walked, so swapping two rows of a thousand moved the 998
  rows between them; it now moves two. Measured on the `js-framework-benchmark` swap
  workload: 179ms to 32ms
* `Value::new` and `Value::set` no longer deep-copy the payload when nothing is listening
  for `Value::add_event`
* A node is computed at most once per change, whatever the shape of the graph. A `get`
  during propagation refreshes a stale parent before returning, so unequal path lengths and
  conditionally read parents no longer recompute a join once per path, and a subscriber
  cannot observe a value assembled from a half-updated graph
* Recomputing a node with many dependencies is no longer quadratic in their number
* A compute closure reading the same value repeatedly records it once
* A `Vec<u8>` arriving from wasm is copied out of linear memory before use. It was decoded as
  a view onto the memory block, which the caller frees before the decoded value is read, so
  the bytes could be reused or grown away from underneath it
* `vertigo build` no longer fails wasm optimization with *"memory.copy operations require
  bulk memory operations"* - the WASM features enabled by default for
  `wasm32-unknown-unknown` are now passed to `wasm-opt` explicitly, because `strip = true`
  removes the `target_features` section it would otherwise read them from
* A server-rendered `LazyCache` no longer re-requests on hydration
* `keyed_computed_list` no longer reports a read after removal when a row is simply removed.

[reactive-graph]: https://docs.rs/vertigo/latest/vertigo/guides/reactive_graph/index.html
[invariants]: https://docs.rs/vertigo/latest/vertigo/reactive/invariants/index.html
[collections]: https://docs.rs/vertigo/latest/vertigo/guides/collection_key_and_list_renderers/index.html

## 0.12.0 - 2026-07-01

### Added

* `LazyListCache` for fetched lists (optimized for CRUD operations)
* `WsCollection` - server-pushed reactive collections over a WebSocket
* `dom!` macro now supports passing children to components
* `on_change_file` attribute for file inputs
* `on_intersect` attribute with `IntersectionEvent`
* `Driver::request_delete`, `Driver::request_patch`, `Driver::request_put` methods

### Fixed

* Partially fixed SVG tags rendering, for full fix trace [#539]
* `TwClass` in dynamic attribute (`dyn:tw={my_tw_class}`)
* `dom!` now correctly handles block of statements with attribute name inferred from variable name in the last statement
* `BTreeMap` deserialization now accepts objects `{...}` as well.
* Treat mixed inline elements as preformatted to preserve whitespace in HTML serialization

## 0.11.4 - 2026-04-18

### Changed

* READMEs fixed.

## 0.11.3 - 2026-04-18

### Added

* `rust_decimal::Decimal` support to `JsJson` (feature `rust_decimal`) [#528]

### Fixed

* Watch script added to non-HTML responses [#532]

### Changed

* websocket: Switch to JsJson for messages

## 0.11.3 - 2026-04-07

### Added

* `#[js_json(skip)]` attribute to skip fields in `AutoJsJson` macro
* `AutoMap::for_each` method for iterating over key-value pairs
* Impl `Add` and `AddAssign` for `Computed<Css>` and `Computed<Tw>`

### Changed

* Allow to pass `AttrGroup` down if one component mounts another inner component [#505]
* Warn if `tw!` macro or `tw=` attribute is used in `#main` macro body [#510]
* In `AutoJsJson` macro the `stringify` attribute now also works for `Option<T>` (where `T` implements `Display` and `FromStr`)

### Fixed

* Allow tailwind classes to be used in component `tw` attribute [#516]
* CSS parse error of comment in animation frames [#522]
* "jsJsonGetSize: Unknown type function" during JS API access [#490]
* `#[store]` macro panics instead of showing a helpful error message

## 0.11.1 - 2026-03-03

### Fixed

* Do not include tailwind output if no `tw` attribute or `tw!` macro is used.

### Changed

* Stabilized `Resource` type (removed nightly `Try` trait) so stable rust can now be used.

### Internals

* Reactored workspace (unified examples and metadata)

## 0.11.0 - 2026-02-27

### Added

* Use `tailwindcss` binary directly (no need for npm)
* Compile-time validation for missing tailwind classes
* Cloneable `TwClass`
* Bearer token as `Computed`
* Fine-grained reactivity when rendering lists
* vertigo-cli: Support compiling on Windows
* vertigo-cli: `--locked` parameter for `build` command

### Changed

* Rust edition 2024
* Allow for external use of DomFragment log (for writing tests)

## 0.10.1 - 2026-01-18

### Added

* Added `stringify` field attribute to `AutoJsJson` to allow stringifying non-`JsJson` types (f. ex. foreign types)
* Added `chrono::NaiveDateTime` support to `JsJson`

### Fixed

* Fixed premature drop in Computed's context

## 0.10.0 - 2026-01-03

### Added

* vertigo-cli: `--threads` parameter for `serve` command, which allows to specify number of threads to use for processing requests
  (defaults to 2 in `watch` mode, and number of CPU cores in `serve` mode)
* vertigo-cli: Added `vertigo_install` function to allow bundling with custom actix-web back-end
* vertigo-cli: Added `--template` parameter for `new` command to allow creating fullstack or frontend project

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
