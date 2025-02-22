use fantoccini::{ClientBuilder, Locator};
use std::time::Duration;
use vertigo_cli::{build, serve, BuildOpts, CommonOpts, ServeOpts};

#[tokio::test]
#[ignore]
async fn basic() {
    // Go to project root
    let _ = std::env::set_current_dir("..");

    // Build basic test site
    let opts = BuildOpts {
        common: CommonOpts {
            dest_dir: "./build".to_string(),
        },
        inner: build::BuildOptsInner {
            package_name: Some("vertigo-test-basic".to_string()),
            public_path: "/build".to_string(),
            disable_wasm_opt: false,
            wasm_run_source_map: true,
        },
    };

    println!("Running site build");

    let ret = build::run(opts);

    assert!(ret.is_ok());

    use tokio::sync::oneshot;
    let (sender, receiver) = oneshot::channel::<i32>();

    println!("Spawning vertigo serve");

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        let opts = ServeOpts {
            common: CommonOpts {
                dest_dir: "./build".to_string(),
            },
            inner: serve::ServeOptsInner {
                host: "127.0.0.1".into(),
                port: 5555,
                proxy: vec![],
                env: vec![],
            },
        };

        handle.block_on(async {
            tokio::select! {
                _ = serve::run(opts, None) => { 1 }
                _ = receiver => { 2 }
            }
        });
    });

    println!("Sleeping for a second waiting for vertigo-cli to start");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("Starting fantoccini");

    let c = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await.expect("failed to connect to WebDriver");

    println!("Opening site");

    c.goto("http://127.0.0.1:5555/")
        .await.expect("goto failed");

    let url = c.current_url()
        .await.expect("current_url failed");

        assert_eq!(url.as_ref(), "http://127.0.0.1:5555/");

    println!("Wait for DOM regeneration by WASM");

    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("Find row numer 2");

    // Find "row-2"
    c.find(Locator::Id("row-2")).await.expect("find row-2 failed");

    // Heatup
    c.find(Locator::Id("generate")).await.expect("heatup: find generate button failed")
        .click().await.expect("heatup: click generate failed");
    c.find(Locator::Id("clear")).await.expect("heatup:  find clear button failed")
        .click().await.expect("heatup: click clear failed");

    // *** Div1 test ***
    let start = std::time::Instant::now();

    // click "Generate"
    c.find(Locator::Id("generate")).await.expect("div: find generate button failed")
        .click().await.expect("div: click generate failed");

    let click_time = start.elapsed();

    println!("div: Generate took {} ms", click_time.as_millis());

    c.find(Locator::Id("row-9999")).await.expect("div: find row-9999 failed");

    let row999_time = start.elapsed();

    println!("div: Row 9999 found {} ms after click", row999_time.as_millis());

    // Change mode
    c.find(Locator::Id("clear")).await.expect("find clear button failed")
        .click().await.expect("click clear failed");
    c.find(Locator::Id("mode_div4")).await.expect("find mode_div4 button failed")
        .click().await.expect("click mode_div4 failed");

    // *** Div4 test ***
    let start = std::time::Instant::now();

    // click "Generate"
    {
        c.find(Locator::Id("generate")).await.expect("div4: find generate button failed")
            .click().await.expect("div4: click generate failed");

        let click_time = start.elapsed();

        println!("div4: Generate took {} ms", click_time.as_millis());

        c.find(Locator::Id("row-9999")).await.expect("div4: find row-9999 failed");

        let row999_time = start.elapsed();

        println!("div4: Row 9999 found {} ms after click", row999_time.as_millis());
    }

    // click "Generate" again
    {
        c.find(Locator::Id("generate")).await.expect("div4: find generate button failed")
            .click().await.expect("div4-2: click generate failed");

        let click_time = start.elapsed();

        println!("div4-2: Generate took {} ms", click_time.as_millis());

        c.find(Locator::Id("row-9999")).await.expect("div4-2: find row-9999 failed");

        let row999_time = start.elapsed();

        println!("div4-2: Row 9999 found {} ms after click", row999_time.as_millis());
    }

    println!("Closing browser");

    c.close().await.expect("close failed");

    sender.send(1).unwrap();

    println!("Sleeping for a second waiting for vertigo-cli to stop");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
