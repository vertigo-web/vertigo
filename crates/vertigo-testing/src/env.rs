//! What a suite runs against. [`Core`] is the part every environment has: the settings, the
//! address of the app, the work directory, an HTTP client and a chromedriver shared by the
//! tests. A test crate wraps it in its own environment, which adds whatever its app needs (a
//! backend, a database) and implements [`Env`].

use std::{
    net::TcpListener,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use thirtyfour::{
    CapabilitiesHelper, ChromiumLikeCapabilities, DesiredCapabilities, LoggingPrefsLogLevel,
    WebDriver,
    extensions::query::ElementPollerWithTimeout,
    manager::{BrowserKind, WebDriverManager},
};

/// Time limit of element queries and of the `Ctx::wait_*` helpers.
pub const WAIT: Duration = Duration::from_secs(10);
pub const POLL: Duration = Duration::from_millis(100);

/// The environment of one suite: started once, before the first test that needs it, shared by
/// all the tests of the suite and shut down after the last one.
///
/// Implementations can write `async fn` for both methods.
pub trait Env: Deref<Target = Core> + Send + Sync + Sized + 'static {
    fn start(suite: &'static str) -> impl Future<Output = Result<Self>>;

    /// Stops what `start` started. The process exits right after it, without running
    /// destructors, so a container left here stays running.
    fn shutdown(&self) -> impl Future<Output = ()>;
}

/// Settings from the `E2E_*` environment variables.
#[derive(Debug, Clone)]
pub struct Settings {
    pub workspace_root: PathBuf,
    pub target_dir: PathBuf,
    /// `E2E_RELEASE`: builds in release mode.
    pub release: bool,
    /// `E2E_WASM_OPT`: runs `wasm-opt` on the apps, as a shipped build has it. Off by default:
    /// it takes long and changes nothing a functional test looks at.
    pub wasm_opt: bool,
    /// `E2E_SKIP_BUILD`: uses the last build.
    pub skip_build: bool,
    /// `E2E_HEADLESS=0` shows the browser window.
    pub headless: bool,
    /// `E2E_CHROMEDRIVER`: chromedriver to use instead of downloading it.
    pub chromedriver: Option<PathBuf>,
    /// `E2E_WEBDRIVER`: URL of a running WebDriver (`http://localhost:9515`) to open the
    /// sessions with, instead of a chromedriver started by the harness.
    pub webdriver: Option<String>,
}

impl Settings {
    /// `manifest_dir` is the `CARGO_MANIFEST_DIR` of the e2e crate, which lives directly in
    /// the root of its workspace.
    pub fn from_env(manifest_dir: &str) -> Result<Self> {
        let workspace_root = Path::new(manifest_dir)
            .parent()
            .with_context(|| format!("{manifest_dir} is not inside a workspace"))?
            .to_path_buf();

        let target_dir = match std::env::var_os("CARGO_TARGET_DIR") {
            Some(dir) => workspace_root.join(dir),
            None => workspace_root.join("target"),
        };

        Ok(Self {
            workspace_root,
            target_dir,
            release: flag("E2E_RELEASE", false),
            wasm_opt: flag("E2E_WASM_OPT", false),
            skip_build: flag("E2E_SKIP_BUILD", false),
            headless: flag("E2E_HEADLESS", true),
            chromedriver: std::env::var_os("E2E_CHROMEDRIVER").map(PathBuf::from),
            webdriver: std::env::var("E2E_WEBDRIVER").ok(),
        })
    }

    pub fn profile(&self) -> &'static str {
        if self.release { "release" } else { "debug" }
    }

    /// `target/e2e/<profile>`, where the apps are built to.
    pub fn build_dir(&self) -> PathBuf {
        self.target_dir.join("e2e").join(self.profile())
    }

    /// `target/e2e/<suite>`: logs and artifacts of failed tests.
    pub fn work_dir(&self, suite: &str) -> PathBuf {
        self.target_dir.join("e2e").join(suite)
    }
}

fn flag(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => parse_flag(&value),
        Err(_) => default,
    }
}

fn parse_flag(value: &str) -> bool {
    !matches!(value, "" | "0" | "false" | "no")
}

/// How the Chrome sessions are set up, apart from what every app gets.
#[derive(Debug, Clone)]
pub struct ChromeConfig {
    pub window_size: (u32, u32),
    /// `--lang`, which also decides `Accept-Language`.
    pub lang: &'static str,
    /// Additional command line arguments.
    pub args: &'static [&'static str],
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            window_size: (1400, 1000),
            lang: "en-US",
            args: &[],
        }
    }
}

/// The part of the environment every suite has. An [`Env`] dereferences to it.
pub struct Core {
    pub suite: &'static str,
    pub settings: Settings,
    /// `http://127.0.0.1:<port>`, without a trailing slash.
    pub base_url: String,
    /// `target/e2e/<suite>`: logs and artifacts of failed tests.
    pub work_dir: PathBuf,
    pub http: reqwest::Client,
    chrome: ChromeConfig,
    webdriver: Arc<WebDriverManager>,
}

impl Core {
    /// Prepares the work directory, removing what the previous run left in `artifacts`.
    pub fn new(
        suite: &'static str,
        settings: Settings,
        base_url: String,
        chrome: ChromeConfig,
    ) -> Result<Self> {
        let work_dir = settings.work_dir(suite);
        std::fs::create_dir_all(&work_dir)
            .with_context(|| format!("can't create {}", work_dir.display()))?;
        // Only the failures of this run belong there
        let _ = std::fs::remove_dir_all(work_dir.join("artifacts"));

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            suite,
            webdriver: webdriver_manager(&settings),
            settings,
            base_url,
            work_dir,
            http,
            chrome,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// A new Chrome session. Each browser test gets its own.
    pub async fn new_browser(&self) -> Result<WebDriver> {
        let mut caps = DesiredCapabilities::chrome();
        if self.settings.headless {
            caps.set_headless()?;
        }
        let (width, height) = self.chrome.window_size;
        caps.add_arg(&format!("--window-size={width},{height}"))?;
        caps.add_arg(&format!("--lang={}", self.chrome.lang))?;
        for arg in [
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-search-engine-choice-screen",
        ]
        .iter()
        .chain(self.chrome.args)
        {
            caps.add_arg(arg)?;
        }
        // After a login form is used, the password manager offers to save the password (and
        // warns about it having leaked). Its bubble takes the clicks meant for the page.
        caps.add_experimental_option(
            "prefs",
            serde_json::json!({
                "credentials_enable_service": false,
                "profile.password_manager_enabled": false,
                "profile.password_manager_leak_detection": false,
            }),
        )?;
        caps.add_arg("--disable-features=PasswordLeakDetection")?;
        // The console is checked after each test
        caps.set_browser_log_level(LoggingPrefsLogLevel::All)?;
        // Leaves `window.alert`/`confirm` open for the test to answer
        caps.set("unhandledPromptBehavior", "ignore")?;

        match &self.settings.webdriver {
            Some(url) => WebDriver::builder(url, caps)
                .poller(poller())
                .await
                .with_context(|| format!("can't start Chrome through the WebDriver at {url}")),
            None => self
                .webdriver
                .launch(caps)
                .await
                .context("can't start Chrome through chromedriver"),
        }
    }
}

/// A port nothing listens on at the moment, for a server the environment starts.
pub fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// Element queries wait up to [`WAIT`], checking every [`POLL`].
fn poller() -> Arc<ElementPollerWithTimeout> {
    Arc::new(ElementPollerWithTimeout::new(WAIT, POLL))
}

fn webdriver_manager(settings: &Settings) -> Arc<WebDriverManager> {
    let mut builder = WebDriverManager::builder().match_local().poller(poller());
    if let Some(chromedriver) = &settings.chromedriver {
        builder = builder.driver_binary(BrowserKind::Chrome, chromedriver);
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::parse_flag;

    #[test]
    fn flags() {
        for off in ["", "0", "false", "no"] {
            assert!(!parse_flag(off), "{off:?}");
        }
        for on in ["1", "true", "yes"] {
            assert!(parse_flag(on), "{on:?}");
        }
    }
}
