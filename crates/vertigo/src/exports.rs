use crate::{
    DomNode, JsJson,
    dev::LongPtr,
    driver_module::{
        api::{api_arguments, api_command_wasm, api_fetch_cache, api_server_handler},
        driver::get_driver,
        get_driver_dom,
        init_env::init_env,
    },
};

/// Starting point of the app (used by [vertigo::main] macro, which is preferred)
pub fn start_app(init_app: fn() -> DomNode) {
    init_env();

    api_fetch_cache().init_cache();

    let root_view = init_app();

    get_driver().set_root(root_view);

    get_driver_dom().flush_dom_changes();
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
