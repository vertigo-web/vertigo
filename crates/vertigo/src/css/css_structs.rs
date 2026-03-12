use std::ops::{Add, AddAssign};

use crate::Computed;

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

impl Add<Css> for Computed<Css> {
    type Output = Computed<Css>;

    fn add(self, rhs: Css) -> Self::Output {
        self.map(move |left| left.extend(rhs.clone()))
    }
}

impl Add<Computed<Css>> for Computed<Css> {
    type Output = Computed<Css>;

    fn add(self, rhs: Computed<Css>) -> Self::Output {
        Computed::from({
            let left = self.clone();
            let right = rhs.clone();
            move |ctx| left.get(ctx) + right.get(ctx)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Css, CssGroup};
    use crate::{Computed, Value, transaction};

    #[test]
    fn computed_css_add_css() {
        let base = Css::str("color: red;");
        let extra = Css::str("background: blue;");

        let value = Value::new(base.clone());
        let comp: Computed<Css> = value.to_computed();
        let result = comp + extra.clone();

        transaction(|ctx| {
            let css = result.get(ctx);
            assert_eq!(
                css.groups,
                vec![
                    CssGroup::CssStatic {
                        value: "color: red;"
                    },
                    CssGroup::CssStatic {
                        value: "background: blue;"
                    }
                ]
            );
        });
    }

    #[test]
    fn computed_css_add_computed_css() {
        let base = Css::str("color: red;");
        let extra = Css::str("background: blue;");

        let value1 = Value::new(base.clone());
        let value2 = Value::new(extra.clone());

        let comp1: Computed<Css> = value1.to_computed();
        let comp2: Computed<Css> = value2.to_computed();

        let result = comp1 + comp2;

        transaction(|ctx| {
            let css = result.get(ctx);
            assert_eq!(
                css.groups,
                vec![
                    CssGroup::CssStatic {
                        value: "color: red;"
                    },
                    CssGroup::CssStatic {
                        value: "background: blue;"
                    }
                ]
            );
        });
    }
}
