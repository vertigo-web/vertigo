# vertigo-testing

Harness for end-to-end browser tests of [vertigo](https://crates.io/crates/vertigo) apps - a reactive Real-DOM library with SSR for Rust.

[![crates.io](https://img.shields.io/crates/v/vertigo-testing)](https://crates.io/crates/vertigo-testing)
[![Documentation](https://docs.rs/vertigo-testing/badge.svg)](https://docs.rs/vertigo-testing)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/vertigo-testing.svg)

* Builds the apps with the vertigo-cli library and serves them with SSR, as `vertigo serve` does,
  or runs a backend of your own.
* Starts the environment once per suite and gives every test its own Chrome session
  ([thirtyfour](https://crates.io/crates/thirtyfour)), downloading a matching chromedriver.
* Helpers that know how a vertigo app behaves - waiting for the app to take over the
  server-rendered page, for a text, a path or a value, the hydration report.
* Fails a test on browser console errors and, on any failure, saves a screenshot, the page source
  and the console log to `target/e2e/<suite>/artifacts`.
* A libtest-compatible runner: filters, `--list`, `--test-threads`, `--ignored`.

## Usage

Keep the tests in a crate of their own, directly in the root of the workspace (e.g. `e2e/`).
Each test binary is one suite:

```toml
[dev-dependencies]
vertigo-testing = "0.13"

[[test]]
name = "app"
path = "tests/app.rs"
harness = false
```

```rust
use std::ops::Deref;

use vertigo_testing::{
    ChromeConfig, Core, Env, Settings, browser_tests, build, prelude::*, serve::Server,
};

struct AppEnv {
    core: Core,
    server: Server,
}

impl Env for AppEnv {
    async fn start(suite: &'static str) -> Result<Self> {
        let settings = Settings::from_env(env!("CARGO_MANIFEST_DIR"))?;
        let build_dir = settings.build_dir().join("my_app");
        build::vertigo_app(&settings, "my_app", &build_dir).await?;
        let server = Server::start(&build_dir).await?;
        let core = Core::new(suite, settings, server.base_url.clone(), ChromeConfig::default())?;
        Ok(Self { core, server })
    }

    async fn shutdown(&self) {
        self.server.stop().await;
    }
}

impl Deref for AppEnv {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.core
    }
}

type Ctx = vertigo_testing::Ctx<AppEnv>;

async fn counter(ctx: Ctx) -> Result<()> {
    ctx.open("/").await?;
    ctx.find(By::Id("increment")).await?.click().await?;
    ctx.wait_for_text("Counter: 1").await
}

fn main() {
    run_suite("app", browser_tests![counter]);
}
```

```sh
cargo test --test app
```

## Settings

| Variable | Effect |
| --- | --- |
| `E2E_RELEASE=1` | Build the apps in release mode |
| `E2E_WASM_OPT=1` | Run `wasm-opt` on the apps, as a shipped build has it |
| `E2E_SKIP_BUILD=1` | Use the last build |
| `E2E_HEADLESS=0` | Show the browser window |
| `E2E_CHROMEDRIVER=<path>` | Use this chromedriver instead of downloading one |
| `E2E_WEBDRIVER=<url>` | Use a running WebDriver (e.g. `http://localhost:9515`) |

See the [vertigo repository](https://github.com/vertigo-web/vertigo) for more information.
