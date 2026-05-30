/// Structure passed as a parameter to the callback on the `on_intersect` event.
///
/// Mirrors a single `IntersectionObserverEntry` from the browser
/// IntersectionObserver API. The bounding box fields describe the observed
/// element's `boundingClientRect` (in CSS pixels) at the time the callback
/// fired.
#[derive(Clone, Debug)]
pub struct IntersectionEvent {
    /// `true` when the observed element intersects the root.
    pub is_intersecting: bool,
    /// Ratio of the element that is visible within the root (`0.0`..=`1.0`).
    pub intersection_ratio: f64,
    /// `boundingClientRect.top` of the observed element, in px.
    pub bounding_top: f64,
    /// `boundingClientRect.bottom` of the observed element, in px.
    pub bounding_bottom: f64,
    /// `boundingClientRect.height` of the observed element, in px.
    pub bounding_height: f64,
}

impl std::fmt::Display for IntersectionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntersectionEvent={}", self.is_intersecting)
    }
}
