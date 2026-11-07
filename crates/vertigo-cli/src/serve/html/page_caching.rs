use vertigo::dev::SsrFetchRequest;

use super::fetch_url::FetchTarget;
use crate::serve::response_state::ResponseState;

/// Who else a rendered page may be given to, going by what its render fetched. The response says
/// it to the caches on the way (nginx, a CDN) and to the browser's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageCaching {
    /// A fetch went to the server's own origin, which gets the browser's cookies, or the render
    /// read a cookie
    varies_on_cookie: bool,
    /// The render read the languages of the browser
    varies_on_languages: bool,
    /// A fetch carried somebody's credentials: a bearer token, or the browser's cookies
    personal: bool,
}

impl PageCaching {
    /// Notes a fetch sent while rendering the page. `cookie` is what goes out with the fetches to
    /// the own origin, if anything does.
    pub fn fetched(
        &mut self,
        request: &SsrFetchRequest,
        target: &FetchTarget,
        cookie: Option<&str>,
    ) {
        let own_origin = matches!(target, FetchTarget::Local(_));

        self.varies_on_cookie |= own_origin;
        self.personal |= request.has_authorization() || (own_origin && cookie.is_some());
    }

    /// Notes that the render read a cookie the app may read on the server.
    pub fn read_cookie(&mut self) {
        self.varies_on_cookie = true;
    }

    /// Notes that the render read the languages of the browser.
    pub fn read_languages(&mut self) {
        self.varies_on_languages = true;
    }

    /// Adds the headers to the response with the page.
    ///
    /// A personal page is `no-store` rather than `no-cache`, which would still let the browser
    /// keep it on disk.
    ///
    /// `Vary: cookie` goes on any page that fetched from the own origin, also one rendered
    /// without cookies, as that's the one a cache could give to somebody with them: hydration
    /// takes the data in the page at its word, and they'd see the page of a guest. Reading a
    /// cookie or the languages is `Vary` alone: f. ex. the language picked isn't private, and the
    /// page may still be shared by those who picked the same.
    pub fn apply(self, response: &mut ResponseState) {
        if self.personal {
            response
                .headers
                .insert("cache-control".into(), "private, no-store".into());
        }

        let vary = [
            (self.varies_on_cookie, "cookie"),
            (self.varies_on_languages, "accept-language"),
        ]
        .into_iter()
        .filter_map(|(varies, header)| varies.then_some(header))
        .collect::<Vec<_>>();

        if !vary.is_empty() {
            response.headers.insert("vary".into(), vary.join(", "));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use actix_web::http::StatusCode;
    use vertigo::{
        FetchMethod,
        dev::{SsrFetchRequest, SsrFetchRequestBody},
    };

    use super::{FetchTarget, PageCaching};
    use crate::serve::response_state::ResponseState;

    const COOKIE: Option<&str> = Some("session=abc");

    fn request(token: Option<&str>) -> SsrFetchRequest {
        SsrFetchRequest {
            method: FetchMethod::GET,
            url: "/api/me".to_string(),
            headers: (token.iter())
                .map(|token| ("Authorization".to_string(), format!("Bearer {token}")))
                .collect::<BTreeMap<_, _>>(),
            body: SsrFetchRequestBody::None,
        }
    }

    fn local() -> FetchTarget {
        FetchTarget::Local("http://127.0.0.1:8080/api/me".to_string())
    }

    fn external() -> FetchTarget {
        FetchTarget::External("https://api.example.com/me".to_string())
    }

    /// The headers of the page, but `content-type`
    fn headers(caching: PageCaching) -> HashMap<String, String> {
        let mut response = ResponseState::html(StatusCode::OK, "<html></html>");
        caching.apply(&mut response);
        response.headers.remove("content-type");
        response.headers
    }

    fn after(fetches: &[(Option<&str>, FetchTarget, Option<&str>)]) -> HashMap<String, String> {
        let mut caching = PageCaching::default();
        for (token, target, cookie) in fetches {
            caching.fetched(&request(*token), target, *cookie);
        }
        headers(caching)
    }

    fn expected(cache_control: Option<&str>, vary: Option<&str>) -> HashMap<String, String> {
        [("cache-control", cache_control), ("vary", vary)]
            .into_iter()
            .filter_map(|(name, value)| Some((name.to_string(), value?.to_string())))
            .collect()
    }

    const PRIVATE: Option<&str> = Some("private, no-store");
    const VARY: Option<&str> = Some("cookie");

    #[test]
    fn a_page_without_fetches_says_nothing() {
        assert_eq!(after(&[]), expected(None, None));
    }

    #[test]
    fn cookies_never_go_to_another_origin() {
        assert_eq!(after(&[(None, external(), COOKIE)]), expected(None, None));
    }

    #[test]
    fn a_guests_page_varies_on_cookies() {
        assert_eq!(after(&[(None, local(), None)]), expected(None, VARY));
    }

    #[test]
    fn cookies_sent_to_own_origin_make_it_personal() {
        assert_eq!(after(&[(None, local(), COOKIE)]), expected(PRIVATE, VARY));
    }

    #[test]
    fn a_token_makes_it_personal_on_any_origin() {
        assert_eq!(
            after(&[(Some("secret"), external(), None)]),
            expected(PRIVATE, None)
        );
    }

    #[test]
    fn reading_a_cookie_varies_on_cookies_but_is_not_personal() {
        let mut caching = PageCaching::default();
        caching.read_cookie();

        assert_eq!(headers(caching), expected(None, VARY));
    }

    #[test]
    fn reading_the_languages_varies_on_them() {
        let mut caching = PageCaching::default();
        caching.read_languages();

        assert_eq!(headers(caching), expected(None, Some("accept-language")));
    }

    #[test]
    fn vary_lists_all_that_was_read() {
        let mut caching = PageCaching::default();
        caching.read_languages();
        caching.read_cookie();

        assert_eq!(
            headers(caching),
            expected(None, Some("cookie, accept-language"))
        );
    }

    #[test]
    fn one_personal_fetch_is_enough() {
        assert_eq!(
            after(&[
                (None, external(), None),
                (Some("secret"), external(), None),
                (None, external(), None),
            ]),
            expected(PRIVATE, None)
        );
    }
}
