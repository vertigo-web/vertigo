//! Renders 10 000 rows in a real browser, in two modes, and prints how long it took.
//!
//! Timed from Rust, around the WebDriver commands, so the numbers include a round trip or two
//! besides the rendering. Printed, never asserted. Runs on the harness of `vertigo-testing`:
//!
//! ```text
//! cargo test --package fantoccini-tests --test basic -- --ignored
//! ```
//!
//! The harness starts a chromedriver of its own; `E2E_WEBDRIVER=http://localhost:9515` uses a
//! running one instead, and `E2E_HEADLESS=0` shows the window. Fantoccini, which this suite
//! ran on before, opened a visible window, so compare with older numbers using that.

use std::{ops::Deref, time::Instant};

use vertigo_testing::{
    ChromeConfig, Core, Env, Settings, browser_tests, build, prelude::*, serve::Server,
};

struct BasicEnv {
    core: Core,
    server: Server,
}

impl Env for BasicEnv {
    async fn start(suite: &'static str) -> Result<Self> {
        let mut settings = Settings::from_env(env!("CARGO_MANIFEST_DIR"))?;
        // What is timed is what ships
        settings.release = true;
        settings.wasm_opt = true;

        let build_dir = settings.build_dir().join("basic");
        build::vertigo_app(&settings, "vertigo-test-basic", &build_dir).await?;
        let server = Server::start(&build_dir).await?;

        let chrome = ChromeConfig {
            // What a chromedriver session gets without capabilities, as the fantoccini one this
            // suite ran on before - so its timings stay comparable
            window_size: (1050, 1000),
            ..Default::default()
        };
        let core = Core::new(suite, settings, server.base_url.clone(), chrome)?;

        Ok(Self { core, server })
    }

    async fn shutdown(&self) {
        self.server.stop().await;
    }
}

impl Deref for BasicEnv {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.core
    }
}

type Ctx = vertigo_testing::Ctx<BasicEnv>;

async fn basic(ctx: Ctx) -> Result<()> {
    ctx.open("/").await?;
    // The app has started, but Chrome still optimizes its wasm in the background. Two seconds
    // let that finish, so what is timed below is the optimized code.
    tokio::time::sleep(Duration::from_secs(2)).await;

    ctx.find(By::Id("row-2")).await?;

    // Heatup
    click(&ctx, "generate").await?;
    click(&ctx, "clear").await?;

    // *** Div test ***
    let start = Instant::now();

    click(&ctx, "generate").await?;
    println!("div: Generate took {} ms", start.elapsed().as_millis());

    ctx.find(By::Id("row-9999")).await?;
    println!(
        "div: Row 9999 found {} ms after click",
        start.elapsed().as_millis()
    );

    // Change mode
    click(&ctx, "clear").await?;
    click(&ctx, "mode_div4").await?;

    // *** Div4 test ***
    let start = Instant::now();

    click(&ctx, "generate").await?;
    println!("div4: Generate took {} ms", start.elapsed().as_millis());

    ctx.find(By::Id("row-9999")).await?;
    println!(
        "div4: Row 9999 found {} ms after click",
        start.elapsed().as_millis()
    );

    // "Generate" again, timed from the same start as the first one: cumulative, as it always
    // was, so the numbers stay comparable
    click(&ctx, "generate").await?;
    println!("div4-2: Generate took {} ms", start.elapsed().as_millis());

    ctx.find(By::Id("row-9999")).await?;
    println!(
        "div4-2: Row 9999 found {} ms after click",
        start.elapsed().as_millis()
    );

    Ok(())
}

async fn click(ctx: &Ctx, id: &str) -> Result<()> {
    ctx.find(By::Id(id)).await?.click().await?;
    Ok(())
}

fn main() {
    // Run on request only (`--ignored`), like the other browser suites: it needs a browser.
    let tests = browser_tests![basic]
        .into_iter()
        .map(|test| TestCase {
            ignored: true,
            ..test
        })
        .collect();

    run_suite("basic", tests);
}
