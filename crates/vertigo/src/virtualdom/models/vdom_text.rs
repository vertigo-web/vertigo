/// Virtual DOM node that represents a text.
///
/// Usually not used directly.
#[derive(Debug, Clone)]
pub struct VDomText {
    pub value: String,
}

impl VDomText {
    pub fn new<T: Into<String>>(value: T) -> VDomText {
        VDomText { value: value.into() }
    }
}
