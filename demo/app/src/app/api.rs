use vertigo::get_driver;

/// Base URL for one of the public APIs this demo reads, overridable at serve time.
///
/// The defaults are the real services, so running the demo normally is unchanged. The
/// override exists for the browser test in `tests/demo`, which points them at the local
/// stand-ins in `demo/server/src/stub_api.rs` rather than depending on someone else's uptime:
///
/// ```text
/// vertigo serve --env api_todo=http://127.0.0.1:5559/todo
/// ```
pub fn api_base(name: &str, default: &str) -> String {
    get_driver()
        .env(name)
        .unwrap_or_else(|| default.to_string())
}
