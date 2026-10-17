use std::{collections::HashMap, sync::Arc};

use vertigo::JsJson;

use super::html::convert_to_jsjson;

#[derive(Clone)]
pub struct RequestState {
    pub url: String,
    pub env: Arc<HashMap<String, String>>,
    /// The languages of the request's `Accept-Language`, the most preferred first.
    pub languages: Arc<Vec<String>>,
    /// The cookies of the request the app may read, by name; see
    /// [`MountConfigBuilder::ssr_cookie`](super::MountConfigBuilder::ssr_cookie).
    pub cookies: Arc<HashMap<String, String>>,
}

impl RequestState {
    pub fn env(&self, name: impl Into<String>) -> Option<String> {
        let name = name.into();
        let value = self.env.get(&name)?;

        Some(value.clone())
    }

    /// The cookie as `cookie_get` gives it the app: empty if the app may not read it, or the
    /// request has none of that name.
    pub fn cookie(&self, name: &str) -> String {
        self.cookies.get(name).cloned().unwrap_or_default()
    }

    /// The cookie as `cookie_get_json` gives it the app in the browser: parsed, or `null` when it
    /// is empty or isn't JSON.
    pub fn cookie_json(&self, name: &str) -> JsJson {
        match self.cookies.get(name) {
            Some(value) if !value.is_empty() => serde_json::from_str(value)
                .map(convert_to_jsjson)
                .unwrap_or(JsJson::Null),
            _ => JsJson::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use vertigo::{JsJson, JsJsonNumber, JsObject};

    use super::RequestState;

    fn state(cookies: &[(&str, &str)]) -> RequestState {
        let cookies: HashMap<String, String> = (cookies.iter())
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        RequestState {
            url: "/".to_string(),
            env: Arc::default(),
            languages: Arc::default(),
            cookies: Arc::new(cookies),
        }
    }

    #[test]
    fn a_cookie_the_app_may_not_read_is_empty() {
        let state = state(&[("lang", "pl")]);

        assert_eq!(state.cookie("lang"), "pl");
        assert_eq!(state.cookie("session"), "");
    }

    #[test]
    fn a_json_cookie_is_parsed_and_one_that_isnt_json_is_null() {
        let state = state(&[("json", r#"{"a":1}"#), ("text", "plain"), ("empty", "")]);

        let mut object = JsObject::new();
        object.insert("a".to_string(), JsJson::Number(JsJsonNumber(1.0)));
        assert_eq!(state.cookie_json("json"), JsJson::Object(object));
        for name in ["text", "empty", "missing"] {
            assert_eq!(state.cookie_json(name), JsJson::Null, "{name}");
        }
    }
}
