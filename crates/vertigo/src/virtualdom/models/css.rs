use crate::get_driver;

/// Css chunk, represented either as static or dynamic string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssGroup {
    // &str - can be used as id using which we can find particular rule
    CssStatic { value: &'static str },
    // string in this case, is a key to hashmap with the class name
    CssDynamic { value: String },
}

/// CSS styles definition for Virtual DOM.
///
/// Consists of a vector of css chunks which can be extended.
///
/// ```rust
/// use vertigo::{Css, CssGroup, html};
///
/// let blue_text = Css::str("color: blue");
/// let black_background = Css::str("background: black");
///
/// let my_styles = Css::str("
///     font-family: courier;
///     font-size: 160%
/// ")
///     .extend(blue_text)
///     .extend(black_background);
///
/// let element = html! { <div css={my_styles} /> };
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Css {
    pub groups: Vec<CssGroup>,
}

impl Css {
    pub fn str(value: &'static str) -> Self {
        Self {
            groups: vec![CssGroup::CssStatic { value }],
        }
    }

    pub fn string(value: String) -> Self {
        Self {
            groups: vec![CssGroup::CssDynamic { value }],
        }
    }

    #[must_use]
    pub fn push_str(mut self, value: &'static str) -> Self {
        self.groups.push(CssGroup::CssStatic { value });
        self
    }

    pub fn push_string(&mut self, value: String) {
        self.groups.push(CssGroup::CssDynamic { value })
    }

    #[must_use]
    pub fn extend(mut self, new_css: Self) -> Self {
        for item in new_css.groups {
            self.groups.push(item);
        }

        self
    }

    pub fn convert_to_string(&self) -> String {
        get_driver().get_class_name(self)
    }
}
