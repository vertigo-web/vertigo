use crate::{
    DomNode, JsJson,
    dev::LongPtr,
    driver_module::{
        api::{api_arguments, api_command_wasm, api_server_handler, close_fetch_cache},
        driver::get_driver,
        get_driver_dom,
        init_env::init_env,
    },
};

/// Starting point of the app (used by [vertigo::main] macro, which is preferred)
pub fn start_app(init_app: fn() -> DomNode) {
    init_env();
    mount(init_app)
}

/// What [`start_app`] does, minus the `fn` pointer - so a test can drive it with a closure
/// that captures - and minus `init_env`. The logger and the panic hook it installs are
/// process-wide, so a test mounting with them would take the logger from tests capturing logs,
/// and hide the panic messages of every test after it.
pub(crate) fn mount(init_app: impl FnOnce() -> DomNode) {
    // Before the transaction, deliberately: `get_driver()` is what registers the flush hook,
    // and the transaction below only suppresses mid-build flushes if it is the outermost one.
    let driver = get_driver();

    // The mount has to reach the browser as a single DOM batch. Hydration gets one shot, and
    // it starts its walk from `<body>` - which is created late, after the whole `<head>`
    // subtree and after anything the app built before reaching its top-level `dom!`.
    // Without this transaction the first `Computed::subscribe` inside the tree closes an
    // outermost transaction of its own, fires the flush hook, and ships a partial batch
    // with no `<html>`/`<head>`/`<body>` in it.
    driver.transaction(|_| {
        let root_view = init_app();
        driver.set_root(root_view);
    });

    // `flush_watch` - and so `when_connect` - runs *after* the hooks, so anything it queued is
    // still sitting in the buffer. A no-op when the buffer is empty.
    get_driver_dom().flush_dom_changes();

    // The page the server rendered is built again by now. What the server fetched for it would
    // be stale for a cache created from here on.
    close_fetch_cache();
}

/// Mounts an app that calls `create` while it renders, and returns what it created - as a
/// component does when it's part of the page from the start.
#[cfg(test)]
pub(crate) fn in_first_render<T>(create: impl FnOnce() -> T) -> T {
    use crate as vertigo;

    let mut created = None;
    mount(|| {
        created = Some(create());
        vertigo::dom! { <div /> }
    });

    // Release the tree while the stores it drops into are still alive - see
    // `Driver::take_root`. Dropping it at thread teardown aborts the process.
    drop(get_driver().take_root());

    let Some(created) = created else {
        unreachable!("mount renders the app")
    };
    created
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub fn vertigo_export_handle_url(url_ptr: u64) -> u64 {
    let url_ptr = LongPtr::from(url_ptr);
    let url = api_arguments().get_by_long_ptr(url_ptr);

    let JsJson::String(url) = url else {
        panic!("string expected");
    };

    let response = api_server_handler().handler(&url);
    response.to_ptr_long().get_long_ptr()
}

//------------------------------------------------------------------------------------------------------------------
// Internals below
//------------------------------------------------------------------------------------------------------------------

// Methods for memory allocation

#[doc(hidden)]
#[unsafe(no_mangle)]
pub fn vertigo_export_alloc_block(size: u32) -> u64 {
    api_arguments().alloc(size).get_long_ptr()
}

#[doc(hidden)]
#[unsafe(no_mangle)]
pub fn vertigo_export_free_block(long_ptr: u64) {
    let long_ptr = LongPtr::from(long_ptr);
    api_arguments().free(long_ptr);
}

// Callbacks gateways

#[doc(hidden)]
#[unsafe(no_mangle)]
pub fn vertigo_export_wasm_command(value_long_ptr: u64) -> u64 {
    let value_long_ptr = LongPtr::from(value_long_ptr);
    let value = api_arguments().get_by_long_ptr(value_long_ptr);

    let response = api_command_wasm().command_from_js(value);
    response.to_ptr_long().get_long_ptr()
}
