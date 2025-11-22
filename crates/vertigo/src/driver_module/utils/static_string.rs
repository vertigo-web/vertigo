use std::borrow::{Borrow, Cow};
use std::fmt::Display;

use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl JsJsonDeserialize for StaticString {
    fn from_json(
        _context: crate::JsJsonContext,
        json: crate::JsJson,
    ) -> Result<Self, crate::JsJsonContext> {
        if let JsJson::String(value) = json {
            return Ok(value.into());
        }

        Err(JsJsonContext::new(format!(
            "Expected String, received={}",
            json.typename()
        )))
    }
}

impl JsJsonSerialize for StaticString {
    fn to_json(self) -> crate::JsJson {
        JsJson::String(self.as_str().to_string())
    }
}
