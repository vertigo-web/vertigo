use super::js_json_struct::JsJson;
use super::{JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

pub static NAIVE_DATE_FORMAT: &str = "%Y-%m-%d";

impl JsJsonSerialize for chrono::DateTime<chrono::Utc> {
    fn to_json(self) -> JsJson {
        self.to_rfc3339().to_json()
    }
}

impl JsJsonDeserialize for chrono::DateTime<chrono::Utc> {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let datetime_str = String::from_json(context.clone(), json)?;
        chrono::DateTime::parse_from_rfc3339(&datetime_str)
            .map_err(|err| {
                let message = ["DateTime parsing failed: ", &err.to_string()].concat();
                context.add(message)
            })
            .map(|dt| dt.to_utc())
    }
}

impl JsJsonSerialize for chrono::NaiveDate {
    fn to_json(self) -> JsJson {
        self.format(NAIVE_DATE_FORMAT).to_string().to_json()
    }
}

impl JsJsonDeserialize for chrono::NaiveDate {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let datetime_str = String::from_json(context.clone(), json)?;
        chrono::NaiveDate::parse_from_str(&datetime_str, NAIVE_DATE_FORMAT).map_err(|err| {
            let message = ["DateTime parsing failed: ", &err.to_string()].concat();
            context.add(message)
        })
    }
}
