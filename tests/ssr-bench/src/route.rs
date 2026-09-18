//! The pages, and the paths they answer on.

use std::fmt::Display;

/// One page shape each, so a regression can be attributed to a phase rather than to "SSR".
///
/// Deliberately not derived from a menu: nothing in this app navigates, and the server is
/// the only thing that ever asks for a route.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Route {
    /// The shell and wrapper every other page also renders, with nothing repeated inside.
    ///
    /// Every other page is this plus N of one thing, which is what makes `t(page) −
    /// t(/tiny)` and `commands(page) − commands(/tiny)` meaningful without anyone having to
    /// know what the shell costs.
    #[default]
    Tiny,
    Wide,
    Deep,
    DeepIndent,
    Text,
    TextPlain,
    Attrs,
    Css,
    Table,
    Roundtrip,
    NotFound,
}

impl Route {
    /// Every page the benchmark renders, in report order. `NotFound` is absent: it is where
    /// a typo lands, not something to measure.
    pub const ALL: &'static [Route] = &[
        Self::Tiny,
        Self::Wide,
        Self::Deep,
        Self::DeepIndent,
        Self::Text,
        Self::TextPlain,
        Self::Attrs,
        Self::Css,
        Self::Table,
        Self::Roundtrip,
    ];

    pub fn path(&self) -> &'static str {
        match self {
            Self::Tiny => "/",
            Self::Wide => "/wide",
            Self::Deep => "/deep",
            Self::DeepIndent => "/deep-indent",
            Self::Text => "/text",
            Self::TextPlain => "/text-plain",
            Self::Attrs => "/attrs",
            Self::Css => "/css",
            Self::Table => "/table",
            Self::Roundtrip => "/roundtrip",
            Self::NotFound => "/not-found",
        }
    }

    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" => Self::Tiny,
            "/wide" => Self::Wide,
            "/deep" => Self::Deep,
            "/deep-indent" => Self::DeepIndent,
            "/text" => Self::Text,
            "/text-plain" => Self::TextPlain,
            "/attrs" => Self::Attrs,
            "/css" => Self::Css,
            "/table" => Self::Table,
            "/roundtrip" => Self::Roundtrip,
            _ => Self::NotFound,
        }
    }
}

impl From<String> for Route {
    /// The raw path, not `route_from_public`.
    ///
    /// `get_driver().route_from_public` is what an app with a mount point would use, but
    /// server-side it is a no-op that costs an `is_browser()` round trip - and this app is
    /// only ever rendered by the benchmark, which asks for the mount-point-stripped path
    /// the way `vertigo_handler` does. Paying for the round trip here would put a constant
    /// on every route that has nothing to do with the page.
    fn from(url: String) -> Self {
        Route::new(url.as_str())
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `new` and `path` are two hand-written lists of the same thing. A typo in either
    /// would send a benchmarked route to `NotFound`, and the run would happily report
    /// timings for the wrong page.
    #[test]
    fn every_route_round_trips_through_its_path() {
        for route in Route::ALL {
            assert_eq!(
                &Route::new(route.path()),
                route,
                "{route:?} has the path {:?}, which does not route back to it",
                route.path()
            );
        }
    }

    #[test]
    fn every_route_has_its_own_path() {
        for (n, route) in Route::ALL.iter().enumerate() {
            for other in &Route::ALL[n + 1..] {
                assert_ne!(route.path(), other.path(), "{route:?} and {other:?}");
            }
        }
    }
}
