<!-- markdownlint-configure-file { "no-duplicate-heading": { "siblings_only": true } } -->

<!-- markdownlint-disable-next-line first-line-h1 -->
## Unreleased

### Added

* `on_focus_in` and `on_focus_out` on elements: the element, or one inside it, got the focus or
  lost it - the DOM's `focusin` and `focusout`, which come from the children too, unlike
  `on_focus` and `on_blur`. The focus going from one element inside to another comes as
  `on_focus_out` and then `on_focus_in`, before the browser shows anything, so f. ex. the buttons
  of a box show while the keyboard is on any of them.
* `Driver::remember_focus`: the element with the focus is given it back when the `DropResource`
  is dropped, if the focus got lost by then - f. ex. a dialog gives it back to the button that
  opened it once it's closed, without scrolling the page to it. The focus moved to another
  element meanwhile stays there, by a Tab or a click that closed a popup in its `on_blur` too.
* `Driver::local_storage_get`, `local_storage_set` and `local_storage_remove`: the browser's
  `localStorage`, kept for the page's origin across visits. Unlike a cookie it isn't sent to the
  server and holds megabytes, f. ex. for a draft kept. The server renders the page without it, so
  a page reads it once the browser took the page over. A browser that refuses it to the page, or
  has no room left, keeps nothing and tells why in the console. Off the browser, as in an app's
  tests, it is a map of the thread.
* `audio`: sound with Web Audio. `Sound` is a short sound, fetched and decoded once, played at
  once when asked, over itself and anything else, at a volume of its own. `Music` streams as it
  plays: played, paused, stopped, looped, with a volume of its own, and `Music::on_end` tells it
  played to its end. `set_volume` and `set_muted` are of all of it. A browser lets a page make
  sound only once the user clicked, pressed a key or touched it: music asked to play before starts
  then, a sound played before isn't heard, and `allowed` tells when sound goes through. The
  browser's code for it is a chunk of its own, loaded when a page first makes a sound.
* Fullscreen: `DomElementRef::request_fullscreen` shows an element on the whole screen, in the
  listener of a click or a key, `Driver::exit_fullscreen` leaves it, and `hook_fullscreen` on
  elements tells when the element is shown there and no longer - also left by the browser on Esc.
* Pointer lock: `DomElementRef::request_pointer_lock` locks the mouse's pointer to an element, in
  the listener of a click or a key, `Driver::exit_pointer_lock` lets it go, and
  `hook_pointer_lock` on elements tells when it's locked and let go - also by the browser on Esc,
  which the page doesn't hear. `PointerEvent::movement_x` and `movement_y` tell how far the
  pointer moved since its last event, all there is of its moves while it's locked, f. ex. for
  a game to look around with the mouse.
* `hook_window_focus` and `hook_visibility` on elements: the window got the focus or lost it,
  and the page was shown or hidden, f. ex. for a game to let go of the keys it holds, whose key-ups
  go elsewhere meanwhile, and to pause.
* `on_key_up` and `hook_key_up`: a key let go, on the element with the focus or anywhere on the
  page, as `on_key_down` and `hook_key_down` tell a key pressed. Both get a `KeyboardEvent`, whose
  `repeat` tells a press repeated by the key held down from the first one, so a game acts once a
  press and goes on moving while a key is held.
* `hook_pointer_down_outside` on elements: a pointer pressed anywhere on the page but on the
  element and what is in it, wherever that lies, f. ex. to close a popup. The press goes on to
  what it fell on, so one click beside a menu closes it and starts what it does there.
* `webgl::WebGl`: WebGL 2 on a `<canvas>` from Rust - buffers, vertex arrays, programs and the
  locations of their uniforms, textures from pixels, and the state of drawing - with handles that
  let their objects go when dropped. The calls of a transaction go to the browser at its end, with the
  changes to the page, so a frame is drawn by calls made in `Driver::on_animation_frame`. Nothing
  is asked back while drawing: a shader that doesn't compile is told in the console. The bitmap is
  as big as the canvas on the screen; `WebGl::size` and `WebGl::on_resize` tell its size, and a
  scene drawn once draws again in `on_resize` without the canvas showing it cleared. A context the
  browser loses is asked back, and `WebGl::on_restore` makes the objects again. Instanced drawing
  (`vertex_attrib_divisor`, `draw_arrays_instanced`, `draw_elements_instanced`) draws many objects
  with a call or two, their matrices worked out in Rust and sent in one buffer. Textures take
  images from URLs too (`texture_image_url`), put in when the browser has them, and
  `WebGl::on_image_load` tells when, to draw again. `pixel_store` reads images flipped or with their
  colors premultiplied by alpha, so mipmaps of see-through pictures don't darken their edges, an
  image from a URL as it was asked for. `depth_mask` and `blend_func_separate` draw see-through
  objects over the others. `WebGl::on_unavailable` tells a browser without WebGL 2, so the app
  can draw another way. `WebGlOptions::bitmap_scale` draws on fewer pixels than the canvas has on
  the screen, stretched over it, for a slow GPU, and `WebGl::set_bitmap_scale` changes it on the
  context the canvas has, its objects kept. A canvas taken off the page loses its context at once,
  so what it held on the GPU goes then, not when the browser collects the canvas. The browser's code
  for it is in a chunk with WebGPU's, loaded when a page first uses either. The demo draws a cube
  turning with it.
* `Driver::on_requested_frame`: a callback for the next frame, run once each time the
  `FrameRequest` it gives asks for it, however often that is before the frame - for a picture
  drawn anew only when something changes.
* vertigo-testing: `Ctx::preserve_drawing_buffers` has the WebGL contexts made from then on keep
  what was drawn, so a test reads the picture back without the app keeping it for tests.
* `webgpu::WebGpu`: WebGPU on a `<canvas>` from Rust, shaped as [wgpu](https://docs.rs/wgpu) but
  cut down - buffers, 2D textures from pixels, samplers, WGSL shader modules, render and compute
  pipelines with the layouts WebGPU finds in their shaders, bind groups, and render and compute
  passes - with handles that let their objects go when dropped. The calls of a transaction go to
  the browser at its end, as those of `WebGl` do, and the passes of a batch to the GPU in one
  command buffer, after what `write_buffer` and `write_texture` wrote before them. The browser
  asks for a GPU device with the first calls, which wait for it; where there is no WebGPU,
  `WebGpu::on_unavailable` tells it and the canvas is still free for `WebGl`. A depth texture as
  big as the canvas (`DepthTarget::Canvas`) is made again when the canvas changes its size, which
  `WebGpu::size` and `WebGpu::on_resize` tell. A device lost is asked for again, and
  `WebGpu::on_restore` makes the objects again. A shader that doesn't compile, or anything WebGPU
  finds wrong, is told in the console. The demo draws the cube of its WebGL tab with it.
* `webgpu`: textures of images from URLs (`WebGpu::create_texture_from_url`, told by
  `WebGpu::on_image_load`), the bind groups with them taking the image when it comes; textures of
  more levels (`TextureDescriptor::mip_level_count`, `Size::mip_levels`) whose smaller images
  `WebGpu::generate_mipmaps` makes, as WebGPU has no way of its own; bind group and pipeline
  layouts of their own (`WebGpu::create_bind_group_layout`, `WebGpu::create_pipeline_layout`), so
  a bind group serves many pipelines, and `RenderPipeline::get_bind_group_layout` for those
  WebGPU finds; a multisampled canvas with smooth edges (`ColorAttachment::canvas_multisampled`,
  `MultisampleState`), its depth texture of as many samples; reads of buffers and of pixels
  (`WebGpu::read_buffer`, `WebGpu::read_pixels`), f. ex. of what a compute pass computed; and
  `WebGpuOptions::bitmap_scale` with `WebGpu::set_bitmap_scale`, as WebGL's. The demo's WebGPU
  tab draws its cube smooth, a picture
  from a file, and numbers doubled by a compute pass, all read back by its e2e test.

### Changed

* `vertigo build` keeps the paths of the machine it runs on out of a release's wasm. rustc
  writes the path of a source file wherever code may panic or log, and for a crate from the
  registry and the standard library's generic code that is a path under the home directory of
  whoever builds, f. ex. `/home/jane/.cargo/registry/src/…/chrono-0.4.42/src/format/parse.rs`,
  sent to every visitor. They are mapped away with `--remap-path-prefix` (`~`, `cargo/registry`,
  `/rustc/<commit>`), on top of the flags the project gives rustc itself wherever it gives
  them. A debug build, as `vertigo watch` makes, keeps the paths for a debugger. The first
  release build after is a full one, as rustc's flags changed.
* `wasm_run.js` is a bootstrap of 0.6 KB with brotli (1.4 KB without) instead of 12.8 KB: all of
  vertigo a page downloads and runs before it has loaded. What wasm talks to the browser through
  is the runtime, a chunk of its own (12.6 KB with brotli) asked for together with the wasm once
  the page has loaded, neither waiting for the other. So the browser's code, which grows with
  what vertigo can do, is no longer downloaded while the page loads, next to what it shows first.
  With `vertigo serve --wasm-preload` the runtime comes while the page loads, as the wasm does.
  `vertigo build` puts it next to `wasm_run.js`, named by its content, so a build copied without
  its directories (`COPY build/*` in a Dockerfile) still starts. A vertigo-cli from before
  doesn't copy it, so its apps don't start: update it together with vertigo.
* vertigo-cli, vertigo-testing, the fullstack template and the demo server build actix-web
  without its `http2` feature, so the server speaks HTTP/1.1 only. Since actix-web 4.19 a server
  with `http2` answers HTTP/2 cleartext (h2c) on `bind`, and actix's HTTP/2 runs on h2 0.3, which
  has no fix for RUSTSEC-2026-0258. An app serving HTTP/2 itself, not behind a proxy, adds
  `http2` to the features of its own actix-web.
* `draw` on a canvas works nothing out during SSR: the server doesn't compute the drawing nor
  turn it into bytes for a page that has no pictures in its canvases, so an app needs no
  `is_server()` check around it.
* `KeyDownEvent` is `KeyboardEvent`, the event of a key let go too, with a new field `repeat`.
  `KeyDownEvent` stays as its other name, so code naming it compiles, but a struct literal of it,
  f. ex. in a test, needs `repeat` or `..Default::default()`, which it now has.
* The browser code drawing on canvases is a chunk of its own, loaded the first time a page needs
  it: `wasm_run.js` is 2.5 KB smaller with brotli (14.2 KB to 11.7 KB) for every app, and an app
  with canvases downloads the chunk (3.3 KB) when the first drawing comes, or meanwhile with its
  wasm when the server sent the page with a canvas. `vertigo build` puts it next to
  `wasm_run.js`, named by its content. A vertigo-cli from before doesn't copy it, so its apps draw
  nothing on canvases.
* A frame of `Driver::on_animation_frame` comes into wasm as a call of its own, with its time as a
  number, and each batch of changes to the page goes to the browser as the bytes it is encoded to,
  through an import of its own (`dom_bulk_update`) - not as commands encoded and decoded both ways.
  A frame drawing little costs about 9 µs less. vertigo-cli of this version is needed to serve
  apps of it: its SSR gives the wasm the new import.

## 0.14.5 - 2026-11-21

### Added

* `Drawing::draw_image`: a part of an image drawn into a rectangle of the drawing, with a new
  `Rect`. The browser loads each image once.
* `Driver::preload_image`: loads an image in the background for drawings to come.
* `Drawing::fill_pattern` and `stroke_pattern`: an image repeated, lying on the drawing.
* `Drawing::fill_linear_gradient` and `stroke_linear_gradient`, with color stops.
* `Drawing::fill_radial_gradient` and `fill_conic_gradient`, and their `stroke_` ones.
* `Drawing::append`: draws another drawing after this one and leaves the styles as they were.
* `Drawing::fill_path`, `stroke_path` and `clip`, with a new `Path2D` (lines, curves, arcs, SVG
  path data) and `FillRule`.
* `Drawing::save`, `restore`, `translate`, `rotate`, `scale`, `transform` and `reset_transform`.
* Text: `Drawing::font`, `font_px`, `text_align`, `text_baseline`, `fill_text` and `stroke_text`.
* `Drawing::line_dash`, `line_dash_px`, `miter_limit`, `stroke_rect` and `clear_rect`.
* `Drawing::composite`, `shadow`, `filter` and `image_smoothing`.
* `Drawing::draw_pixels`: pixels worked out in Rust, f. ex. a heat map, drawn into a rectangle.
* `ClickEvent::position`: where the pointer clicked, as a new `ClickPosition` (CSS pixels from the
  corner of the element with the `on_click`, and its size). `None` for a click from the keyboard.
* `guides::for_agents`: rules for a coding agent writing an app with vertigo.
* README: what is different coming from Leptos, Yew or Dioxus, and how vertigo helps coding
  agents.
* vertigo-cli: `vertigo build` saves its text files compressed with brotli (`name.ext.br`), which
  `serve --precompress` takes instead of compressing them at start. `--precompress false` skips it.
* vertigo-cli: `is_ssr_fetch`: whether a request is the app's fetch during SSR, for a page handler
  of your own.
* vertigo-cli: `vertigo new` puts an `AGENTS.md` and a `CLAUDE.md` in the new project.
  `--no-agent-files` leaves them out.

### Changed

* `dev::command::DriverDomCommand::CanvasDraw` carries the drawing as an `Arc<Vec<u8>>`, shared
  with the `Drawing` instead of copied.
* `Drawing` is `Send` and `Sync`.
* `ViewBox::new` is a `const fn`.
* vertigo-cli: A build file named by its content (`name.HASH.ext`) is served with
  `Cache-Control: public, max-age=31536000, immutable`.
* vertigo-cli: No longer depends on OpenSSL: `--proxy` doesn't go through `actix-proxy` any more.

### Fixed

* The docs of `Drawing::fill_style` promised gradient keywords, which a canvas doesn't take.
* A `ViewBox` without a width and a height above zero, or with a NaN or an infinity, drew the
  drawing anyway instead of nothing.
* vertigo-cli: `--proxy` passed on only the last value of a repeated response header, f. ex.
  `Set-Cookie`.
* vertigo-cli: The fullstack template (`vertigo new -t fullstack`) didn't set `ssr_fetch_base`, so
  the app's fetches of relative URLs failed during SSR.

## 0.14.4 - 2026-11-16

### Breaking

* `AttrGroupValue` lost its per-event variants (`OnClick`, `Draw`, `HookKeyDown`, …): an event
  handler in an `AttrGroup` is `AttrGroupValue::Binding`, made by `AttrGroupValue::on_click` etc.

### Added

* `on_focus` on elements, the pair of `on_blur`.
* `hook_pointer_move`, `hook_pointer_up` and `hook_pointer_cancel` on elements: the pointer events
  of the whole page, with the position from the element with the hook.
* `PointerEvent::release_pointer`: the pointer's events go again to the element under it.
* `on_resize` on elements, with a `ResizeEvent`: the element's drawn size.
* `Drawing::fill_polygon` and `Drawing::stroke_polygon`.
* `Driver::next_paint`: ends once the browser has drawn the page.
* vertigo-cli: `MountConfigBuilder::ssr_fetch_cookie` and `--ssr-fetch-cookie`: only the named
  cookies go out with the SSR fetches to the server's own origin (default: all).

### Performance

* An app spreading an `AttrGroup` carries the code of only the events it uses: about 4 kB less
  gzipped wasm with a few events.

### Fixed

* A file input with `on_change_file` was emptied once its files were read.
* `AutoJsJson` read an integer out of the type's range or with a fraction without an error.
* `AutoJsJson`'s error for a number of the wrong type said `number($name)` instead of the type.
* `AutoJsJson` didn't read a `Vec<u8>` field from a JSON list of numbers, f. ex. from an API.
* The browser sent a `Vec<u8>` in a fetch body, a websocket text message or a JSON cookie as an
  object keyed by index instead of a list of numbers.
* `on_message` got bytes (`Uint8Array`) from a frame as an object keyed by index instead of a list
  of numbers.
* vertigo-testing: On Linux `ChromeConfig::lang` didn't decide `Accept-Language` and
  `navigator.languages`.
* On nightly, a macro error came with follow-up errors from the code the macro made anyway.
* A `LazyCache` or `LazyListCache` with `bearer_auth` didn't pick up its SSR response: hydration
  dropped the part of the page made from it, and it fetched again.
* A `LazyCache` read in the transaction that had just set it gave the value from before.
* Navigation to a fragment inside a closed `<details>` or a `hidden="until-found"` element didn't
  reveal it, so nothing scrolled.
* Hydration closed a `<details>` the browser had opened.
* vertigo-cli: The bearer token of an SSR fetch was in the page source (`data-fetch-cache`).
* vertigo-cli: Pages rendered with someone's credentials, cookies or languages lacked
  `Cache-Control: private, no-store` or `Vary`, so a shared cache could serve them to others.

### Internals

* The macros report errors and warnings without proc-macro-error (unmaintained,
  RUSTSEC-2024-0370), so an app no longer builds syn 1.

## 0.14.3 - 2026-11-06

### Added

* `on_scroll` on elements, with a `ScrollEvent`: where the content is scrolled to and how big it
  and its visible part are.
* `#[store(all = name)]`: for a store with arguments, a function `name` giving all its values.
* `WsQuery::ge` and `WsQuery::le`: inclusive range filters (`>=`, `<=`).
* `Driver::history_push`: goes to an address with a new entry in the browser's history.
* vertigo-cli: `serve::vary_accept_encoding`, a middleware that adds `Vary: accept-encoding` to
  what a back-end's own `Compress` sends uncompressed.
* vertigo-cli: `MountConfigBuilder::precompress_in_release` precompresses in a release build only.

### Changed

* `Router::set`, `Driver::history_push` and `Driver::history_replace` scroll to the address's
  fragment (`/post#comment-5`) once its element is on the page, as a followed link does.
* `vertigo serve --precompress` logs when it starts compressing, in a debug build as a warning.

### Performance

* Less wasm - the demo's is about a tenth smaller after wasm-opt: `#[store]`, `Value`, `Computed`,
  `LazyCache`, `render_value`, transactions and JSON reading keep less code generic over types.

### Fixed

* `#[store]` dropped the attributes of its function: doc comments, `#[cfg]`, etc.
* `include_static!` files and `tw!`/`tw=` classes of a library crate were left out of the bundle
  when cargo took the crate from its cache.
* `vertigo serve` left `Vary: accept-encoding` out of a compressible response sent uncompressed,
  so a cache on the way (nginx, a CDN) could give every browser the first client's version.

## 0.14.2 - 2026-10-29

### Added

* `on_message` on an `<iframe>`: the messages its page sends with `parent.postMessage`.
* `on_media` on a `<video>` or an `<audio>`: its media events, each with the player's state.
* vertigo-cli: `vertigo_files` installs only the build's static files, for a back-end that serves
  the pages itself.

### Fixed

* An e2e test run rebuilt crates like ring every time, because vertigo-cli and vertigo-testing
  passed the test binary's `CARGO_PKG_*` and `CARGO_MANIFEST_*` on to cargo.

### Internals

* vertigo-cli uses wasmtime 50 and parking_lot 0.13.
* ESLint (`just js-lint`) checks the JS driver.

## 0.14.1 - 2026-10-24

### Breaking

* `form=` in `dom!` is the HTML `form` attribute, no longer a shorthand for `on_submit`.

### Added

* `vertigo serve --precompress` (`MountConfigBuilder::precompress`): the build's text files are
  compressed with brotli once, at startup, about a quarter smaller. `vertigo build` shows the gain.
* `on_change_checked` for checkboxes and radio buttons.
* `ClickEvent` carries the modifier keys: `alt_key`, `ctrl_key`, `shift_key`, `meta_key`.

### Fixed

* `checked` from a `Computed` or a `Value` didn't reach a checkbox the user had clicked.
* `vertigo serve --wasm-preload` made the browser download the wasm twice.

## 0.14.0 - 2026-10-21

Websockets report why they closed, carry binary frames and reconnect after a growing wait.
Elements get pointer and wheel events and `<canvas>` drawing, and a page rendered on the server
can use the user's languages and the cookies the app names.

The [0.14.0 release notes][notes-0140] carry the migration steps, the reasoning behind the
breaking changes and an example of each new feature.

### Breaking

* Every event from the browser runs in one transaction, so what it writes reaches the DOM in one
  batch. Until the handler returns, a watched `Computed` of what it wrote and the DOM keep their
  state from before the event.
* `WebsocketMessage::Close` carries a `WebsocketClose` with the close code and reason, and comes
  also after a failed attempt to connect.
* `WebsocketMessage` has a new variant, `Binary`.

### Added

* Default component props: `#[default = <constant>]` on a `#[component]` param, and `{..}` in its
  tag (nightly).
* `WebsocketClose::stop_reconnecting`, to give up on a websocket.
* `Driver::websocket_with_options` with `WebsocketOptions`: the timeout to connect and the waits
  between attempts.
* Binary websocket frames: `WebsocketMessage::Binary` and `WebsocketConnection::send_binary`.
* `--proxy` of `vertigo serve` and `vertigo watch` carries websockets too.
* `Driver::on_animation_frame`: a callback before every frame the browser draws.
* `on_wheel` on elements, with a `WheelEvent`.
* `on_pointer_down`, `on_pointer_move`, `on_pointer_up` and `on_pointer_cancel` on elements, with
  a `PointerEvent`.
* `Driver::languages`: the languages the user prefers, in the browser and during SSR.
* `MountConfigBuilder::ssr_cookie` and `--ssr-cookie` of `vertigo serve` and `vertigo watch`: a
  cookie the app may read during SSR too.
* `ServerState::request_with_headers` with `RequestHeaders`, and `request_accept_language`: the
  request's cookies and `Accept-Language` for SSR.
* `draw` on a `<canvas>`, with a `Drawing` of 2D context operations.

### Changed

* A websocket reconnects after a growing wait (0.5-1 s, doubled up to 15-30 s) instead of every
  3 s.
* `vertigo-testing` needs thirtyfour 0.38 (was 0.37.8), which uses zip 9 instead of 8
  (RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369). It re-exports thirtyfour, so a crate
  that also depends on thirtyfour itself needs 0.38 too.

### Fixed

* A websocket dropped while still connecting connected anyway and stayed open.
* Websockets of a page in the browser's back/forward cache stayed open.
* The driver moved its `<style>` after every batch of DOM changes, so the browser restyled the
  page each time.
* A `when_connect` closure or an `on_after_transaction` callback holding the last handle of
  another node panicked when dropped ("RefCell already borrowed").
* A `render_value` or `render_list` right under `<html>` broke the page after hydration since
  0.13.4.

[notes-0140]: https://github.com/vertigo-web/vertigo/blob/master/docs/releases/0.14.0.md

## 0.13.4 - 2026-10-14

### Changed

* SSR now sends HTML without indentation so hydration performs better.
  `--pretty-html` (`MountConfigBuilder::pretty_html`) brings indentation back.
* SSR fetches and the proxy use rustls 0.23 (was 0.20, RUSTSEC-2024-0336) with the `ring`
  provider, unless the app installed one, and HTTP/1.1 only (h2 0.3, RUSTSEC-2026-0258).

### Fixed

* Hydration moved every node the server sent, so the focus left the field,
  scrolled boxes went back to the top, iframes reloaded.
* The hydration report didn't count server-rendered elements that hydration threw away.
* `LazyListCache` didn't reuse the list fetched during server-side rendering, so the browser
  showed it loading and fetched it again.
* `LazyCache` created after the first render (f. ex. on navigating back to a route) started with
  the value the server fetched for the page, which could be stale by then, instead of fetching it.
* `css!` panicked on an attribute in a pseudo-class argument: `:not([disabled])`.
* `css!` left an expression in the middle of a value unsubstituted: `1px solid {color}`.
* `css!` macro fixes for:

  * pseudo-elements: `::before { ... }`,
  * chained pseudo-classes: `:first-child:hover`,
  * pseudo-classes with arguments: `:not(:last-child)`, `:is(:hover, :focus)`,
  * selector lists: `:hover, :focus`,
  * pseudo-classes after a referenced Css: `[other_css]:hover`,
  * custom properties with digits, uppercase letters or `_`: `--gap-2`,
  * `url(...)` followed by other values: `url('a.png') no-repeat`,
  * keyframe selectors: `from`, `to`, `0%, 100%`, `33.3%`,
  * `@media` without a colon in the query: `@media print`,
  * a semicolon in quotes or parentheses: `content: ';'`, data URI,
  * no semicolon after the last declaration or after a `@media` block.

* A referenced Css block (`[other_css] { ... }`) relied on CSS nesting, now it's a descendant
  selector.
* `vertigo-testing`: with `E2E_HEADLESS=0`, windows of parallel tests took the focus from each
  other, so `Ctx::retype` typed after the old value and drop-downs closed mid-test.

### Performance

* Hydration built its virtual tree in quadratic time - now linear
  (f. ex. ~7k nodes dropped from 130 ms to 27 ms).

## 0.13.3 - 2026-10-10

### Added

* `vertigo-testing` - harness for end-to-end browser tests of vertigo apps.

### Fixed

* Hydration of an empty text no longer claims the following text and removes the element in
  between.
* `vertigo new -t fullstack` names the backend `<project>-be` (kebab-case).

### Internals

* `Justfile` (just 1.46+) replaces `Taskfile.yaml` and `Makefile.toml`.

## 0.13.2 - 2026-10-01

### Added

* SSR can fetch relative URLs (base set by `MountConfigBuilder::ssr_fetch_base`/`--ssr-fetch-base`).
* SSR forwards the browser's cookies to same-origin fetches (`ServerState::request_with_cookie`).
* `LazyCache::refresh()` - refetch without a loading state.
* Links with `rel="external"` are left to the browser.

### Changed

* `on_click` no longer prevents the default action unless `ClickEvent::prevent_default()` is
  called.

### Fixed

* Fetch keeps the HTTP status for non-JSON bodies; bare `text/plain` is recognized as text.
* Links with a fragment scroll to the element instead of to the top.
* Modified clicks (Ctrl/Cmd/Shift/Alt, non-main button), `target` and `download` links are left
  to the browser.
* `mailto:`, `tel:` and `javascript:` links work; same-origin absolute and relative links are
  routed in the app.
* An app with a mount point no longer takes over links outside of it.
* `Driver::route_to_public` and `Driver::public_build_path` give the same path in SSR as in the
  browser.
* SSR HTML preserves text whitespace exactly (incl. `<pre>` and `<textarea>`).
* `Item id not found` when an `autofocus` element was removed before getting focus.

## 0.13.1 - 2026-09-21

### Added

* `vertigo serve` and `vertigo watch` now compress responses (brotli/gzip/zstd, by content
  negotiation). The wasm binary and `wasm_run.js` are then about 3 times smaller.
  Pass `--disable-compression` behind a proxy or CDN that already compresses.
  The fullstack template (`vertigo new -t fullstack`) does the same through
  `App::wrap(Compress::default())`.

### Performance

* Reduced WASM sizes by around 10% by de-genericization.
* Reduces `wasm_run.js` by 9% - dead code removed, and different functions were optimized.
* SSR: Per-request WASM instantiation got about 24% cheaper (see fix below).

### Fixed

* Hydration fail when the app subscribes to a value before the root `dom!` block.
* `--proxy` forwarded the upstream's `Content-Encoding` and `Content-Length` even though `awc`
  had already decoded the body, so proxying to an upstream that compressed produced a response
  no client could decode.
* SSR: WASM instantiation types order problem. Imports are now resolved by name, once at startup.
  Per-request instantiation got about 24% cheaper. A missing or mistyped import is now reported
  at startup, naming the import, instead of per request.

### Internals

* Added tests for the JsJson codec (`jsjson.ts`).

* Added benchmarks:

  * `task ssr-bench` renders a set of pages and reports where the time went,
  * `task hydration-bench` loads a set of server-rendered pages in a real browser and reports
    how long hydration took and how many DOM mutations it performed,
  * `task reactive-bench` runs the reactive-graph benchmark suite,
  * `task dom-bench` runs the end-to-end benchmark suite.

  `task bench` runs all four benchmarks. `task bench-compare` runs all four benchmarks and
  compares them against the baseline.

  See [benchmarks][benchmarks]

* `vertigo-cli` gained an off-by-default `ssr-timings` feature, which is what exposes the
  per-phase breakdown through `ServerState::request_timed`. It compiles to nothing when off

[benchmarks]: https://github.com/vertigo-web/vertigo/blob/master/docs/benchmarks.md

## 0.13.0 - 2026-09-07

The reactive graph was rewritten and keyed list rendering was rebuilt around per-key
`Computed`s. Printing a type into the DOM is now opt-in through `DomDisplay`.

The [0.13.0 release notes][notes-0130] carry the migration steps, the reasoning behind each
breaking change and the mechanism behind the performance numbers. See also the
[reactive graph guide][reactive-graph] and the [collection guide][collections].

### Breaking

* `Computed<T>` requires `T: PartialEq` everywhere, not only for `subscribe`
* a `Computed` recomputes when its dependencies change, not on the next read, and reading
  through `transaction(|ctx| ...)` serves the cached value
* `Value::set` from a compute closure, a `subscribe` callback or a `DropResource` destructor
  is ignored and reported through `log::error!` rather than panicking. Event handlers,
  timers, fetch, `on_after_transaction` and `when_connect` (`create`) are unaffected - see
  the [invariants][invariants]
* `when_connect` and `Value::with_connect` run `create` after the wave that made the node
  watched (and after `on_after_transaction`), and drop the resource after the wave that
  unwatched it, instead of in the middle of a refresh. `create` may write; the destructor
  must only tear down the external handler
* `EmbedDom` is no longer implemented for every `T: ToString`, by value or by reference, and
  `AttrValue` is likewise no longer built from every `T: ToString`. A printable type of your
  own opts in with one line - `impl vertigo::DomDisplay for Route {}` - which covers both
  `<a href={my_route}>` and `<div>{my_route}</div>`. Primitives, the string types and the
  reactive wrappers are unaffected
* `AttrValue` has a new `Static(&'static str)` variant, and `AttrValue::get` returns the new
  `AttrText` instead of `Rc<String>`
* `render_list` takes a `Vec<T>` source, render closure - `&Computed<T>` instead of `&T`
* the render closures of `render_list_memo` and `render_resource_list_memo` receive
  `&Computed<T::Value>`; `Loading` and `Error` render as an empty list
* removed: `Dependencies` - use `transaction` and `on_after_transaction`
* removed: `Computed::subscribe_all` - use `subscribe`
* removed: `ValueSynchronize`, `Value::synchronize`, `LazyCache::synchronize` and
  `CacheValue::synchronize` - use `keyed_computed_list`
* removed: `Collection` and `CollectionModel`, superseded by `keyed_computed_list`

### Added

* A new reactive graph in the `vertigo::reactive` module: transactional, with an equality
  cutoff. See the [guide][reactive-graph]
* `reactive::transaction` and `reactive::on_after_transaction`
* `GraphId` - identity of a `Value` or `Computed`
* `ByRc` - an `Rc` compared by pointer identity, so a handle owning something incomparable
  (a `DropResource`, a callback, a subscription) can still live in a `Computed`
* `RenderValue` - trait form of `render_value` / `render_value_option`, for generic code
* `keyed_computed_list` and `KeyedListItem` - per-key `Computed`s from a reactive list,
  reusing the same `Computed` for a given key across updates (Solid `<For>`-style)
* `MarkerContent` - lets a marker comment report the nodes it keeps in front of itself
* `DomDisplay` - opts a `Display` type of your own into everything vertigo renders. One impl
  covers the value, references to it and its reactive wrappers, in attributes and as text
* `IntoAttrValue` - the attribute-value bound of `DomElement::attr` and the `dom!` macro.
  Blanket over `Into<AttrValue>`, so nothing to implement; it exists for the error message
* `DomElement::attr_static`, emitted by `dom!` when an attribute's name and value are both
  literals - `class="col-md-1"`, `aria-hidden="true"`
* `DomText::new_computed_display`, for `T: ToString`, beside the `T: Into<String>` form

### Changed

* two DOM shape changes, invisible in the browser but visible to a test asserting on
  rendered HTML: every `render_list` row is preceded by an anchor comment node, and
  `<li>{value}</li>` no longer emits a trailing `<!-- v -->` inside the element
* `DomText::new_computed` creates its node with the value already set rather than empty
* `AutoJsJson` encodes a `Vec<u8>` field of an enum variant as `JsJson::Vec` - one
  length-prefixed byte run - which is what it already did for a struct field of that type.
  It previously produced a list with a `JsJson::Number` per byte
* `vertigo new` now writes a `[profile.release]` into the project it generates. The fullstack
  template additionally splits `opt-level` per target through `.cargo/config.toml`, so the
  frontend is built for size and the backend for speed

### Performance

Mechanism and measurement conditions for each of these are in the
[release notes][notes-0130].

* DOM commands travel to the browser in a fast flat binary format
* reordering a keyed list moves only the rows whose order actually changed
* updating one row of a keyed list no longer costs work proportional to list length,
* a reactive value embedded in `dom!` updates its text node in place
* an application that uses no `css!` no longer links the css engine (~13 kB gzipped less)
* the SSR fetch cache decodes on first use rather than at startup, so an application with no
  `LazyCache` drops the decode path entirely (another ~13 kB gzipped less)
* `AutoJsJson` builds an object one field at a time instead of with `BTreeMap::from([..])` -
  1-5% of the wasm, depending on how many types the application derives
* vertigo's own hash maps use FxHash instead of `SipHash-1-3`
* fewer copies in the graph: Value's `new` and `set` skip the deep copy when nothing listens

### Fixed

* a server-rendered `LazyCache` no longer re-requests on hydration
* `render_list` corrupted sibling order when reordering or inserting rows whose root is a
  plain element rather than a `render_value` marker
* moving a row repositions its DOM instead of rebuilding it, so input values, listeners and
  children survive a reorder
* a node is computed at most once per change, whatever the shape of the graph
* a `Vec<u8>` arriving from wasm is copied out of linear memory before use. It was decoded as
  a view onto the memory block, which the caller frees before the decoded value is read, so
  the bytes could be reused or grown away from underneath it
* `vertigo build` no longer fails wasm optimization with *"memory.copy operations require
  bulk memory operations"* - the WASM features enabled by default for
  `wasm32-unknown-unknown` are now passed to `wasm-opt` explicitly
* `keyed_computed_list` no longer reports a read after removal when a row is simply removed
* the bundling macros re-run when they need to, giving faster rebuilds
* `vertigo build` honours a `--target-dir` passed through to cargo

[notes-0130]: https://github.com/vertigo-web/vertigo/blob/master/docs/releases/0.13.0.md
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
