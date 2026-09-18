//! The pages, and the paths they answer on.

use std::fmt::Display;

use vertigo::get_driver;

/// Two groups, and a floor.
///
/// **Clean-match** pages render the same tree on the server and in the browser, so every
/// node can be adopted and the only question is what adopting costs. **Mismatch** pages
/// render deliberately different trees, which is where the two implementations' matching
/// strategies diverge and where the cost of getting it wrong shows up.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Route {
    /// The shell and wrapper alone. The fixed cost of a page load with nothing on it.
    #[default]
    Tiny,

    // -- clean match ----------------------------------------------------------------
    Wide,
    Deep,
    Text,
    Attrs,
    Table,

    // -- mismatch -------------------------------------------------------------------
    /// The browser renders the same rows in a different order.
    MismatchOrder,
    /// The browser wraps each row in two extra levels the server never sent.
    MismatchDepth,
    /// The server puts an attribute on every row that the browser does not want.
    MismatchAttrs,
    /// Each row is several adjacent text nodes, which the server glues into one.
    MismatchText,

    NotFound,
}

impl Route {
    /// Every page the benchmark loads, in report order.
    pub const ALL: &'static [Route] = &[
        Self::Tiny,
        Self::Wide,
        Self::Deep,
        Self::Text,
        Self::Attrs,
        Self::Table,
        Self::MismatchOrder,
        Self::MismatchDepth,
        Self::MismatchAttrs,
        Self::MismatchText,
    ];

    /// The pages whose server and browser trees agree, so hydration should match everything.
    /// `tests.rs` asserts `matched == hydratable` on exactly these.
    pub const CLEAN: &'static [Route] = &[
        Self::Tiny,
        Self::Wide,
        Self::Deep,
        Self::Text,
        Self::Attrs,
        Self::Table,
    ];

    pub fn path(&self) -> &'static str {
        match self {
            Self::Tiny => "/",
            Self::Wide => "/wide",
            Self::Deep => "/deep",
            Self::Text => "/text",
            Self::Attrs => "/attrs",
            Self::Table => "/table",
            Self::MismatchOrder => "/mismatch-order",
            Self::MismatchDepth => "/mismatch-depth",
            Self::MismatchAttrs => "/mismatch-attrs",
            Self::MismatchText => "/mismatch-text",
            Self::NotFound => "/not-found",
        }
    }

    pub fn new(path: &str) -> Route {
        match path {
            "" | "/" => Self::Tiny,
            "/wide" => Self::Wide,
            "/deep" => Self::Deep,
            "/text" => Self::Text,
            "/attrs" => Self::Attrs,
            "/table" => Self::Table,
            "/mismatch-order" => Self::MismatchOrder,
            "/mismatch-depth" => Self::MismatchDepth,
            "/mismatch-attrs" => Self::MismatchAttrs,
            "/mismatch-text" => Self::MismatchText,
            _ => Self::NotFound,
        }
    }

    /// Short label for the report table.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Wide => "wide",
            Self::Deep => "deep",
            Self::Text => "text",
            Self::Attrs => "attrs",
            Self::Table => "table",
            Self::MismatchOrder => "mismatch-order",
            Self::MismatchDepth => "mismatch-depth",
            Self::MismatchAttrs => "mismatch-attrs",
            Self::MismatchText => "mismatch-text",
            Self::NotFound => "not-found",
        }
    }
}

impl From<String> for Route {
    /// Through `route_from_public`, which strips the mount point.
    ///
    /// Not optional, and not an optimisation to skip: the server is handed the path already
    /// stripped by `vertigo_handler`, but the browser reads `window.location.pathname` and
    /// gets the mount point with it. Matching the raw path works server-side and silently
    /// routes every page to `NotFound` in the browser - which renders as the shell, makes the
    /// server's markup unmatchable, and turns the benchmark into a measurement of deleting a
    /// document. The suite mounts at `/hb` and `/hb-off` precisely because two servers cannot
    /// share one mount point, so there is always something to strip.
    fn from(url: String) -> Self {
        Route::new(get_driver().route_from_public(url).as_str())
    }
}

impl Display for Route {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `new` and `path` are two hand-written lists of the same thing. A typo in either would
    /// send a benchmarked route to `NotFound`, and the run would report timings for the
    /// wrong page without saying so.
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
    fn every_route_has_its_own_path_and_slug() {
        for (index, route) in Route::ALL.iter().enumerate() {
            for other in &Route::ALL[index + 1..] {
                assert_ne!(route.path(), other.path(), "{route:?} and {other:?}");
                assert_ne!(route.slug(), other.slug(), "{route:?} and {other:?}");
            }
        }
    }

    /// Every clean-match route is a real route. Otherwise the assertion that uses `CLEAN`
    /// would be checking a page that is never loaded.
    #[test]
    fn clean_routes_are_all_benchmarked() {
        for route in Route::CLEAN {
            assert!(Route::ALL.contains(route), "{route:?} is not in ALL");
        }
    }
}
