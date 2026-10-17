//! The headers of a browser's request that change how the server renders a page, and what the
//! app gets from them during SSR.

use std::collections::HashMap;

/// The headers of a browser's request that change how the server renders a page, for
/// [`ServerState::request_with_headers`](super::ServerState::request_with_headers).
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestHeaders<'a> {
    /// `Cookie`. It goes out with the SSR fetches to the server's own origin, and the cookies
    /// named with [`MountConfigBuilder::ssr_cookie`](super::MountConfigBuilder::ssr_cookie) are
    /// what `get_driver().cookie_get()` gives the app.
    pub cookie: Option<&'a str>,
    /// `Accept-Language`: what `get_driver().languages()` gives the app.
    pub accept_language: Option<&'a str>,
}

/// The language tags of an `Accept-Language` header, the most preferred first: by their weight,
/// and those of the same weight in the order of the header. What the browser doesn't want
/// (`q=0`), the wildcard `*` and what doesn't parse are left out.
///
/// Browsers send the header from the list in `navigator.languages`, so this is the list the
/// browser will tell the app after hydration.
pub fn accept_languages(header: &str) -> Vec<String> {
    let mut weighted: Vec<(f32, &str)> = header.split(',').filter_map(weighted_tag).collect();
    // Stable, so those of the same weight keep the header's order.
    weighted.sort_by(|(left, _), (right, _)| right.total_cmp(left));
    weighted
        .into_iter()
        .map(|(_, tag)| tag.to_string())
        .collect()
}

/// A language range of `Accept-Language` with its weight, unless the browser doesn't want it or
/// it doesn't parse.
fn weighted_tag(range: &str) -> Option<(f32, &str)> {
    let mut parts = range.split(';').map(str::trim);
    let tag = parts.next().filter(|tag| is_language_tag(tag))?;
    let mut weight = 1.0;
    for param in parts {
        let Some((name, value)) = param.split_once('=') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("q") {
            weight = value
                .trim()
                .parse::<f32>()
                .ok()
                .filter(|weight| (0.0..=1.0).contains(weight))?;
        }
    }
    (weight > 0.0).then_some((weight, tag))
}

/// A language tag as browsers send them: letters and digits, in parts joined by `-`.
fn is_language_tag(tag: &str) -> bool {
    tag.split('-')
        .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_alphanumeric()))
}

/// The cookies of a `Cookie` header named in `names`, as the driver reads them in the browser
/// (`Cookies.get`): the value URI-decoded, and a part that isn't one `name=value` left out. Of
/// two with the same name, the first counts, as there.
pub fn readable_cookies(header: &str, names: &[String]) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for cookie in header.split(';') {
        let chunks: Vec<&str> = cookie.trim().split('=').collect();
        let [name, value] = chunks[..] else {
            continue;
        };
        if !names.iter().any(|wanted| wanted == name) || cookies.contains_key(name) {
            continue;
        }
        if let Some(value) = decode_uri_component(value) {
            cookies.insert(name.to_string(), value);
        }
    }
    cookies
}

/// `value` decoded as `decodeURIComponent` decodes it, `None` where that throws: a `%` without two
/// hex digits after it, or bytes that aren't UTF-8.
fn decode_uri_component(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let mut rest = value.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'%' {
            let hex = tail
                .get(..2)
                .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit))?;
            let hex = std::str::from_utf8(hex).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(byte);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::{accept_languages, readable_cookies};

    #[test]
    fn languages_come_the_most_preferred_first() {
        assert_eq!(
            accept_languages("pl-PL,pl;q=0.9,en-US;q=0.8,en;q=0.7"),
            ["pl-PL", "pl", "en-US", "en"]
        );
        assert_eq!(
            accept_languages("en;q=0.5, de, fr;Q=0.9"),
            ["de", "fr", "en"]
        );
    }

    #[test]
    fn languages_of_the_same_weight_keep_the_order_of_the_header() {
        assert_eq!(
            accept_languages("da, en-GB;q=0.8, en;q=0.8"),
            ["da", "en-GB", "en"]
        );
    }

    #[test]
    fn what_the_browser_doesnt_want_or_doesnt_parse_is_left_out() {
        assert_eq!(accept_languages("fr;q=0, *;q=0.1, it"), ["it"]);
        assert_eq!(
            accept_languages(" , ;q=1, en;q=x, de;q=2, <b>, en--US, zh-Hant-TW;level=1"),
            ["zh-Hant-TW"]
        );
        assert!(accept_languages("").is_empty());
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn only_the_cookies_named_are_read() {
        let cookies = readable_cookies("session=secret; lang=en; theme=dark", &names(&["lang"]));

        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies.get("lang").map(String::as_str), Some("en"));
    }

    #[test]
    fn cookies_are_read_as_the_driver_reads_them_in_the_browser() {
        let wanted = names(&["note", "json", "bad", "twice", "plus", "eq"]);
        let cookies = readable_cookies(
            "note=Za%C5%BC%C3%B3%C5%82%C4%87%20g%C4%99%C5%9Bl%C4%85; json=%7B%22a%22%3A1%7D; \
             bad=%E0%A4%A; twice=first; twice=second; plus=a+b; eq=a=b",
            &wanted,
        );

        assert_eq!(cookies["note"], "Zażółć gęślą");
        assert_eq!(cookies["json"], r#"{"a":1}"#);
        // `decodeURIComponent` throws on these, and `Cookies.get` skips a part with two `=`.
        assert!(!cookies.contains_key("bad") && !cookies.contains_key("eq"));
        assert_eq!(cookies["twice"], "first");
        assert_eq!(cookies["plus"], "a+b");
    }
}
