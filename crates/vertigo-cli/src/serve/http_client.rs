use awc::http::Version;
use std::sync::Once;

/// Client for SSR fetches and the proxy.
///
/// awc builds its rustls config with the process-level crypto provider, which rustls can't pick
/// by itself when the app has none or both of its `ring` and `aws-lc-rs` features enabled.
/// Installs `ring` unless the app installed a provider already.
///
/// HTTP/1.1 only: awc's HTTP/2 runs on h2 0.3, which has no fix for RUSTSEC-2026-0258.
pub fn http_client() -> awc::Client {
    static PROVIDER: Once = Once::new();
    PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    awc::Client::builder()
        .max_http_version(Version::HTTP_11)
        .finish()
}
