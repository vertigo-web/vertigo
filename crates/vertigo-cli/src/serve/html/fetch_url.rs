use url::{ParseError, Url};

/// Marks an SSR fetch to the server's own address; `vertigo_handler` answers it with 404
/// instead of rendering a page (which could fetch the same URL again).
pub const SSR_FETCH_HEADER: &str = "x-vertigo-ssr-fetch";

/// Where SSR sends a fetch.
#[derive(Debug, PartialEq, Eq)]
pub enum FetchTarget {
    External(String),
    /// Resolved against `ssr_fetch_base`
    Local(String),
}

impl FetchTarget {
    pub fn url(&self) -> &str {
        match self {
            Self::External(url) | Self::Local(url) => url,
        }
    }
}

/// Resolves a relative URL like the browser would on the rendered page, but with `base` as origin.
pub fn resolve_fetch_url(
    url: &str,
    base: Option<&str>,
    mount_point: &str,
    local_url: &str,
) -> Result<FetchTarget, String> {
    match Url::parse(url) {
        Err(ParseError::RelativeUrlWithoutBase) => {}
        Ok(_) | Err(_) => return Ok(FetchTarget::External(url.to_string())),
    }

    let Some(base) = base else {
        return Err(format!(
            "Relative URL {url:?} can't be fetched during SSR without ssr_fetch_base"
        ));
    };

    let base = Url::parse(base).map_err(|err| format!("Invalid ssr_fetch_base {base:?}: {err}"))?;

    let page = format!(
        "{}/{}",
        mount_point.trim_end_matches('/'),
        local_url.trim_start_matches('/')
    );
    let target = base
        .join(&page)
        .and_then(|page| page.join(url))
        .map_err(|err| format!("Can't resolve {url:?} against {page:?}: {err}"))?;

    // `//other.host/path` is relative too, but points elsewhere
    if target.origin() == base.origin() {
        Ok(FetchTarget::Local(target.into()))
    } else {
        Ok(FetchTarget::External(target.into()))
    }
}

/// Origin under which a server bound to `host:port` reaches itself.
pub fn local_origin(host: &str, port: u16) -> String {
    let host = match host {
        "" | "0.0.0.0" => "127.0.0.1",
        "::" | "[::]" => "[::1]",
        host if host.contains(':') && !host.starts_with('[') => {
            return format!("http://[{host}]:{port}");
        }
        host => host,
    };
    format!("http://{host}:{port}")
}

#[cfg(test)]
mod tests {
    use super::{FetchTarget, local_origin, resolve_fetch_url};

    const BASE: Option<&str> = Some("http://127.0.0.1:8080");

    fn local(url: &str) -> Result<FetchTarget, String> {
        Ok(FetchTarget::Local(url.to_string()))
    }

    #[test]
    fn absolute_url_is_left_as_is() {
        assert_eq!(
            resolve_fetch_url("https://api.example.com/posts?page=2", BASE, "/", "/"),
            Ok(FetchTarget::External(
                "https://api.example.com/posts?page=2".to_string()
            ))
        );
        // no base needed for absolute URLs
        assert_eq!(
            resolve_fetch_url("http://localhost:4444/api", None, "/", "/"),
            Ok(FetchTarget::External(
                "http://localhost:4444/api".to_string()
            ))
        );
    }

    #[test]
    fn absolute_path() {
        assert_eq!(
            resolve_fetch_url("/api/posts?page=2", BASE, "/", "/blog/post/?utm=x"),
            local("http://127.0.0.1:8080/api/posts?page=2")
        );
    }

    #[test]
    fn path_relative_to_page() {
        assert_eq!(
            resolve_fetch_url("comments", BASE, "/", "/blog/post/"),
            local("http://127.0.0.1:8080/blog/post/comments")
        );
        assert_eq!(
            resolve_fetch_url("../list", BASE, "/", "/blog/post"),
            local("http://127.0.0.1:8080/list")
        );
        assert_eq!(
            resolve_fetch_url("?page=2", BASE, "/", "/blog/"),
            local("http://127.0.0.1:8080/blog/?page=2")
        );
    }

    #[test]
    fn mount_point_is_part_of_page_url() {
        assert_eq!(
            resolve_fetch_url("data.json", BASE, "/app", "/post/"),
            local("http://127.0.0.1:8080/app/post/data.json")
        );
        assert_eq!(
            resolve_fetch_url("data.json", BASE, "/app/", "post/"),
            local("http://127.0.0.1:8080/app/post/data.json")
        );
        assert_eq!(
            resolve_fetch_url("/api", BASE, "/app", "/"),
            local("http://127.0.0.1:8080/api")
        );
    }

    #[test]
    fn protocol_relative_url_to_other_host_is_external() {
        assert_eq!(
            resolve_fetch_url("//cdn.example.com/data.json", BASE, "/", "/"),
            Ok(FetchTarget::External(
                "http://cdn.example.com/data.json".to_string()
            ))
        );
    }

    #[test]
    fn relative_url_without_base_is_an_error() {
        let result = resolve_fetch_url("/api/posts", None, "/", "/");
        assert!(
            matches!(&result, Err(error) if error.contains("ssr_fetch_base")),
            "{result:?}"
        );
    }

    #[test]
    fn origin_of_bound_address() {
        assert_eq!(local_origin("127.0.0.1", 4444), "http://127.0.0.1:4444");
        assert_eq!(local_origin("0.0.0.0", 80), "http://127.0.0.1:80");
        assert_eq!(local_origin("localhost", 8080), "http://localhost:8080");
        assert_eq!(local_origin("::", 4444), "http://[::1]:4444");
        assert_eq!(local_origin("::1", 4444), "http://[::1]:4444");
    }
}
