use std::ops::{Add, AddAssign};

/// Css chunk, represented either as static or dynamic string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssGroup {
    // &str - can be used as id using which we can find particular rule
    CssStatic { value: &'static str },
    // string in this case, is a key to hashmap with the class name
    CssDynamic { value: String },
    CssMedia { query: String, rules: Vec<String> },
}

/// CSS styles definition for use in DOM.
///
/// Consists of a vector of css chunks which can be extended.
///
/// ```rust
/// use vertigo::{Css, dom};
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
/// let element = dom! { <div css={my_styles} /> };
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

    /// Extend current Css returning new one.
    #[must_use]
    pub fn extend(mut self, new_css: Self) -> Self {
        for item in new_css.groups {
            self.groups.push(item);
        }

        self
    }

    /// Extend current Css with other Css in-place.
    pub fn extend_inplace(&mut self, new_css: Self) {
        for item in new_css.groups {
            self.groups.push(item);
        }
    }
}

impl Add for Css {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.extend(rhs)
    }
}

impl Add for &Css {
    type Output = Css;

    fn add(self, rhs: Self) -> Self::Output {
        self.clone().extend(rhs.clone())
    }
}

impl Add<&Self> for Css {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        self.extend(rhs.clone())
    }
}

impl Add<Css> for &Css {
    type Output = Css;

    fn add(self, rhs: Css) -> Self::Output {
        self.clone().extend(rhs)
    }
}

impl AddAssign for Css {
    fn add_assign(&mut self, other: Self) {
        self.extend_inplace(other);
    }
}

impl AddAssign<&Self> for Css {
    fn add_assign(&mut self, other: &Self) {
        self.extend_inplace(other.clone());
    }
}

impl AddAssign<&Css> for &mut Css {
    fn add_assign(&mut self, other: &Css) {
        self.extend_inplace(other.clone());
    }
}
