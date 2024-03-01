use std::borrow::{Borrow, Cow};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StaticString(pub Cow<'static, str>);

impl StaticString {
    pub fn as_str(&self) -> &str {
        self.0.borrow()
    }

    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Cow::Borrowed(str) => str.is_empty(),
            Cow::Owned(str) => str.is_empty(),
        }
    }
}

impl From<&'static str> for StaticString {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for StaticString {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl Default for StaticString {
    fn default() -> Self {
        Self(Cow::Borrowed(""))
    }
}

impl Display for StaticString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Cow::Borrowed(data) => f.write_str(data),
            Cow::Owned(data) => f.write_str(data),
        }
    }
}
