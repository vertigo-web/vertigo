//! Bring the demo up in a real browser, and take it down again.
//!
//! Same shape as the other three binaries in this crate: build the wasm package, spawn
//! `vertigo serve` on a thread, connect a WebDriver. What is extra here is the API server -
//! the demo has a backend, and the tabs that use it are the interesting ones.

use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator, elements::Element, error::CmdError};
use serde_json::{Map, json};
use tokio::sync::oneshot;
use vertigo_cli::{BuildOpts, CommonOpts, ServeOpts, build, serve};

/// Distinct from `basic` (5555), `reactive_bench` (5556) and `dom_bench` (5557): cargo may run
/// the test binaries concurrently.
pub const SERVE_PORT: u16 = 5558;
/// The demo's own API - `/api/items`, the two websockets, and the stubs standing in for
/// jsonplaceholder and GitHub.
pub const API_PORT: u16 = 5559;
/// Distinct for the same reason, and because `build::run` wipes its dest dir on entry.
const DEST_DIR: &str = "./build-demo";
const PACKAGE: &str = "vertigo-demo";
const WEBDRIVER: &str = "http://localhost:9515";

/// How long any single "the DOM should say X by now" poll is given.
///
/// Generous, because it also covers a websocket connecting and a fetch round-trip. Nothing
/// here is a timing assertion - a slow machine should be slow, not red.
pub const SETTLE: Duration = Duration::from_secs(15);

/// Poll interval. Short enough that a passing run does not idle, long enough not to hammer the
/// WebDriver session.
const POLL: Duration = Duration::from_millis(100);

pub struct Harness {
    pub client: Client,
    api: vertigo_demo_server::ServerHandle,
    serve_stop: Option<oneshot::Sender<i32>>,
}

impl Harness {
    pub async fn start() -> Harness {
        // Go to project root
        let _ = std::env::set_current_dir("..");

        println!("Starting the demo API server on port {API_PORT}");

        let api = vertigo_demo_server::start_background("127.0.0.1", API_PORT)
            .expect("could not start the demo API server");

        println!("Building {PACKAGE}");

        let opts = BuildOpts {
            common: CommonOpts {
                dest_dir: DEST_DIR.to_string(),
                log_local_time: None,
            },
            inner: build::BuildOptsInner {
                package_name: Some(PACKAGE.to_string()),
                public_path: None,
                // Release, so what is tested is what ships. Debug builds also emit extra
                // `v-component` and `v-css` attributes, which would show up in the class
                // assertions below.
                wasm_opt: Some(true),
                release_mode: Some(true),
                wasm_run_source_map: false,
                cargo_opts: vec![],
            },
        };

        assert!(build::run(opts).is_ok(), "build failed");

        let (serve_stop, receiver) = oneshot::channel::<i32>();

        println!("Spawning vertigo serve on port {SERVE_PORT}");

        let handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            let api = format!("http://127.0.0.1:{API_PORT}");

            let opts = ServeOpts {
                common: CommonOpts {
                    dest_dir: DEST_DIR.to_string(),
                    log_local_time: None,
                },
                inner: serve::ServeOptsInner {
                    host: "127.0.0.1".into(),
                    port: SERVE_PORT,
                    mount_point: "/".to_string(),
                    // The lazy-list tab asks for a relative `/api/items`, so that one has to
                    // arrive same-origin.
                    proxy: vec![("/api".to_string(), format!("{api}/api"))],
                    env: vec![
                        // Websockets go direct: `install_proxy` forwards with `awc` and does
                        // not do upgrade handshakes.
                        (
                            "ws_chat".to_string(),
                            format!("ws://127.0.0.1:{API_PORT}/ws"),
                        ),
                        (
                            "ws_collection".to_string(),
                            format!("ws://127.0.0.1:{API_PORT}/ws-collection"),
                        ),
                        // The two public APIs, pointed at their local stand-ins.
                        ("api_todo".to_string(), format!("{api}/todo")),
                        ("api_github".to_string(), format!("{api}/github")),
                    ],
                    wasm_preload: true,
                    disable_hydration: false,
                    threads: None,
                },
            };

            handle.block_on(async {
                tokio::select! {
                    ret = serve::run(opts, None) => {
                        if let Err(err) = ret {
                            println!("Can't spawn vertigo-cli: {err:?}");
                        }
                    }
                    _ = receiver => {}
                }
            });
        });

        // `serve::run` waits for the port to come free before binding, so how long it takes to
        // start depends on what else was just using it. Poll rather than guess - a fixed sleep
        // here surfaces as ERR_CONNECTION_REFUSED from `goto`, which says nothing useful.
        wait_for_listener(SERVE_PORT).await;

        println!("Connecting to the WebDriver at {WEBDRIVER}");

        let mut capabilities = Map::new();
        // "ignore" leaves a dialog standing so this test can answer it deliberately - the JS
        // API tab opens two alerts and a prompt, and the prompt's text is asserted on. The
        // default, dismiss-and-notify, would swallow them and fail the *next* command
        // instead, which is a confusing way to learn that a click opened a dialog.
        capabilities.insert("unhandledPromptBehavior".to_string(), json!("ignore"));

        let client = ClientBuilder::native()
            .capabilities(capabilities)
            .connect(WEBDRIVER)
            .await
            .expect("failed to connect to WebDriver - is chromedriver running on :9515?");

        let site_url = format!("http://127.0.0.1:{SERVE_PORT}/");
        println!("Opening {site_url}");
        client.goto(&site_url).await.expect("goto failed");

        Harness {
            client,
            api,
            serve_stop: Some(serve_stop),
        }
    }

    pub async fn finish(mut self) {
        println!("Closing the browser");
        self.client.close().await.expect("close failed");

        if let Some(stop) = self.serve_stop.take() {
            stop.send(1).ok();
        }
        self.api.stop(false).await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Block until something is accepting connections on `port`.
async fn wait_for_listener(port: u16) {
    let deadline = std::time::Instant::now() + SETTLE;

    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }

        assert!(
            std::time::Instant::now() < deadline,
            "nothing came up on port {port} within {SETTLE:?}"
        );

        tokio::time::sleep(POLL).await;
    }
}

// --- polling helpers ---------------------------------------------------------------------
//
// Everything the demo does is asynchronous somewhere - a websocket connecting, a fetch
// landing, a `bind_spawn` animation ticking. Every assertion below therefore polls until the
// DOM says what it should, or gives up with a message naming what it was waiting for. There
// are no bare sleeps followed by an assertion.

/// Poll `check` until it returns true, then return. Panics with `what` on timeout.
pub async fn wait_until<F, Fut>(what: &str, mut check: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<bool, CmdError>>,
{
    let deadline = std::time::Instant::now() + SETTLE;

    loop {
        // A transient error is normal while the DOM is being rebuilt underneath us: an element
        // found a moment ago can be stale by the time it is read. Only `Ok(true)` ends this.
        if let Ok(true) = check().await {
            return;
        }

        if std::time::Instant::now() >= deadline {
            panic!("timed out after {SETTLE:?} waiting for: {what}");
        }

        tokio::time::sleep(POLL).await;
    }
}

/// Wait until the whole page's text contains `needle`.
pub async fn wait_for_text(client: &Client, needle: &str) {
    wait_for_body(client, needle, true).await
}

/// Wait until the page's text no longer contains `needle`.
pub async fn wait_for_no_text(client: &Client, needle: &str) {
    wait_for_body(client, needle, false).await
}

async fn wait_for_body(client: &Client, needle: &str, wanted: bool) {
    let deadline = std::time::Instant::now() + SETTLE;

    loop {
        let text = body_text(client).await.unwrap_or_default();

        if text.contains(needle) == wanted {
            return;
        }

        if std::time::Instant::now() >= deadline {
            let verb = if wanted { "contain" } else { "lose" };
            // The page's own text is the only useful thing to say here, and reading it back is
            // how a renamed label or a failed fetch identifies itself.
            panic!(
                "timed out after {SETTLE:?} waiting for the page text to {verb} {needle:?}\n\
                 page text was:\n{text}"
            );
        }

        tokio::time::sleep(POLL).await;
    }
}

/// Wait until `selector` matches exactly `count` elements.
pub async fn wait_for_count(client: &Client, selector: &str, count: usize) {
    wait_until(
        &format!("{count} element(s) matching {selector:?}"),
        || async { Ok(client.find_all(Locator::Css(selector)).await?.len() == count) },
    )
    .await;
}

pub async fn body_text(client: &Client) -> Result<String, CmdError> {
    client.find(Locator::Css("body")).await?.text().await
}

/// Every element matching `selector`, waiting for at least one to exist first.
pub async fn find_all(client: &Client, selector: &str) -> Vec<Element> {
    wait_until(&format!("at least one {selector:?}"), || async {
        Ok(!client.find_all(Locator::Css(selector)).await?.is_empty())
    })
    .await;

    client
        .find_all(Locator::Css(selector))
        .await
        .unwrap_or_default()
}

/// Pull the element reference out of a script's return value.
///
/// A WebDriver wraps a returned DOM node in a single-key object whose key is the spec's web
/// element identifier (`element-6066-…`). Matched by prefix rather than spelled out, so a
/// mistyped constant cannot silently turn every lookup into "not found".
fn element_ref(value: &serde_json::Value) -> Option<String> {
    value
        .as_object()?
        .iter()
        .find(|(key, _)| key.starts_with("element-"))
        .and_then(|(_, id)| id.as_str())
        .map(str::to_string)
}

/// The element matching `selector` whose *own* text is exactly `text`.
///
/// Own text means this element's direct text-node children, and nothing a descendant
/// contributes. That distinction is what makes the demo's nested clickable divs addressable:
/// `<div on_click>"outer click"<br/><button>"Inner click"</button></div>` owns "outer click"
/// while its rendered text is both. Neither `Element::text()` (which is the whole subtree) nor
/// an XPath `text()` (which is the *first* text node, and so misses `"post = " {title}`) says
/// what is wanted here.
///
/// Matched in one script rather than a round-trip per candidate - some tabs put hundreds of
/// divs on the page.
pub async fn find_by_text(client: &Client, selector: &str, text: &str) -> Element {
    const SCRIPT: &str = r#"
        const [selector, wanted] = arguments;
        return Array.from(document.querySelectorAll(selector)).find((node) =>
            Array.from(node.childNodes)
                .filter((child) => child.nodeType === Node.TEXT_NODE)
                .map((child) => child.textContent)
                .join('')
                .trim() === wanted
        ) || null;
    "#;

    let deadline = std::time::Instant::now() + SETTLE;

    loop {
        let result = client
            .execute(SCRIPT, vec![json!(selector), json!(text)])
            .await;
        let found = result.as_ref().ok().and_then(element_ref);

        if let Some(id) = found {
            return Element::from_element_id(client.clone(), id.into());
        }

        if std::time::Instant::now() >= deadline {
            // Says what the page *does* offer, so a renamed label reads as a rename rather
            // than as a mystery.
            let candidates = own_texts(client, selector).await;
            panic!(
                "timed out after {SETTLE:?} looking for a {selector} owning the text {text:?}\n\
                 last script result: {result:?}\n\
                 {selector} elements own: {candidates:?}"
            );
        }

        tokio::time::sleep(POLL).await;
    }
}

/// Every own-text on the page for `selector`, for use in a failure message.
async fn own_texts(client: &Client, selector: &str) -> Vec<String> {
    const SCRIPT: &str = r#"
        const [selector] = arguments;
        return Array.from(document.querySelectorAll(selector)).map((node) =>
            Array.from(node.childNodes)
                .filter((child) => child.nodeType === Node.TEXT_NODE)
                .map((child) => child.textContent)
                .join('')
                .trim()
        );
    "#;

    client
        .execute(SCRIPT, vec![json!(selector)])
        .await
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .filter(|text| !text.is_empty())
        .collect()
}

/// Click the element matching `selector` whose own text is exactly `text`.
pub async fn click_by_text(client: &Client, selector: &str, text: &str) {
    find_by_text(client, selector, text)
        .await
        .click()
        .await
        .unwrap_or_else(|err| panic!("clicking the {selector} reading {text:?} failed: {err}"));
}

/// Replace an input's contents, the way a person would.
///
/// Select-all and backspace rather than `Element::clear`: a WebDriver clear does not reliably
/// reach the page as an `input` event, and everything in this demo learns about a change from
/// one - emptying a field with `clear` looks to the app like nothing happened. The escapes are
/// the WebDriver key codes for Control, NULL (which releases held modifiers) and Backspace.
pub async fn retype(element: &Element, text: &str) {
    element
        .send_keys("\u{E009}a\u{E000}\u{E003}")
        .await
        .expect("clearing the field failed");

    if !text.is_empty() {
        element.send_keys(text).await.expect("send_keys failed");
    }
}
