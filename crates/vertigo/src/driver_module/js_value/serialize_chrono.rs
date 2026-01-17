use super::js_json_struct::JsJson;
use super::{JsJsonContext, JsJsonDeserialize, JsJsonSerialize};

pub static NAIVE_DATE_FORMAT: &str = "%Y-%m-%d";
pub static NAIVE_DATE_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

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

impl JsJsonSerialize for chrono::NaiveDateTime {
    fn to_json(self) -> JsJson {
        self.format(NAIVE_DATE_TIME_FORMAT).to_string().to_json()
    }
}

impl JsJsonDeserialize for chrono::NaiveDateTime {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let datetime_str = String::from_json(context.clone(), json)?;
        chrono::NaiveDateTime::parse_from_str(&datetime_str, NAIVE_DATE_TIME_FORMAT).map_err(
            |err| {
                let message = ["DateTime parsing failed: ", &err.to_string()].concat();
                context.add(message)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize};
    use chrono::{Datelike, Timelike};
    use std::error::Error;

    #[test]
    fn datetime_utc_serialization() -> Result<(), Box<dyn Error>> {
        use chrono::{TimeZone, Utc};

        let dt = Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 45);

        let dt = match dt {
            chrono::LocalResult::Single(dt) => dt,
            _ => return Err("Invalid datetime".into()),
        };

        let json = dt.to_json();

        // Should serialize to RFC3339 format
        if let JsJson::String(s) = json {
            assert!(s.contains("2025-01-15"));
            assert!(s.contains("14:30:45"));
        } else {
            panic!("Expected JsJson::String");
        }

        Ok(())
    }

    #[test]
    fn datetime_utc_deserialization() -> Result<(), Box<dyn Error>> {
        use chrono::{DateTime, Utc};

        let rfc3339_str = "2025-01-15T14:30:45Z";
        let json = JsJson::String(rfc3339_str.to_string());
        let context = JsJsonContext::new("test");

        let result = DateTime::<Utc>::from_json(context, json);
        assert!(result.is_ok());

        let dt = result?;
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);

        Ok(())
    }

    #[test]
    fn datetime_utc_round_trip() -> Result<(), Box<dyn Error>> {
        use chrono::{DateTime, TimeZone, Utc};

        let original = Utc.with_ymd_and_hms(2025, 6, 20, 10, 15, 30).unwrap();
        let json = original.to_json();
        let context = JsJsonContext::new("test");

        let result = DateTime::<Utc>::from_json(context, json);
        assert!(result.is_ok());

        let deserialized = result?;
        assert_eq!(original, deserialized);

        Ok(())
    }

    #[test]
    fn datetime_utc_invalid_format() {
        use chrono::{DateTime, Utc};

        let invalid_str = "not a valid datetime";
        let json = JsJson::String(invalid_str.to_string());
        let context = JsJsonContext::new("test");

        let result = DateTime::<Utc>::from_json(context, json);
        assert!(result.is_err());

        if let Err(err_context) = result {
            assert!(err_context.to_string().contains("DateTime parsing failed"));
        } else {
            panic!("Expected Err");
        }
    }

    #[test]
    fn naive_date_serialization() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2025, 3, 25).ok_or("Invalid date")?;
        let json = date.to_json();

        // Should serialize to YYYY-MM-DD format
        assert_eq!(json, JsJson::String("2025-03-25".to_string()));

        Ok(())
    }

    #[test]
    fn naive_date_deserialization() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDate;

        let date_str = "2025-03-25";
        let json = JsJson::String(date_str.to_string());
        let context = JsJsonContext::new("test");

        let result = NaiveDate::from_json(context, json);
        assert!(result.is_ok());

        let date = result?;
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 25);

        Ok(())
    }

    #[test]
    fn naive_date_round_trip() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDate;

        let original = NaiveDate::from_ymd_opt(2024, 12, 31).ok_or("Invalid date")?;
        let json = original.to_json();
        let context = JsJsonContext::new("test");

        let result = NaiveDate::from_json(context, json);
        assert!(result.is_ok());

        let deserialized = result?;
        assert_eq!(original, deserialized);

        Ok(())
    }

    #[test]
    fn naive_date_invalid_format() {
        use chrono::NaiveDate;

        let invalid_str = "2025/03/25"; // Wrong separator
        let json = JsJson::String(invalid_str.to_string());
        let context = JsJsonContext::new("test");

        let result = NaiveDate::from_json(context, json);
        assert!(result.is_err());

        if let Err(err_context) = result {
            assert!(err_context.to_string().contains("DateTime parsing failed"));
        } else {
            panic!("Expected Err");
        }
    }

    #[test]
    fn naive_datetime_serialization() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2025, 7, 10).ok_or("Invalid date")?;
        let datetime = date.and_hms_opt(18, 45, 30).ok_or("Invalid time")?;
        let json = datetime.to_json();

        // Should serialize to ISO 8601 format (YYYY-MM-DDTHH:MM:SS)
        assert_eq!(json, JsJson::String("2025-07-10T18:45:30".to_string()));

        Ok(())
    }

    #[test]
    fn naive_datetime_deserialization() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDateTime;

        let datetime_str = "2025-07-10T18:45:30";
        let json = JsJson::String(datetime_str.to_string());
        let context = JsJsonContext::new("test");

        let result = NaiveDateTime::from_json(context, json);
        assert!(result.is_ok());

        let datetime = result?;
        assert_eq!(datetime.year(), 2025);
        assert_eq!(datetime.month(), 7);
        assert_eq!(datetime.day(), 10);
        assert_eq!(datetime.hour(), 18);
        assert_eq!(datetime.minute(), 45);
        assert_eq!(datetime.second(), 30);

        Ok(())
    }

    #[test]
    fn naive_datetime_round_trip() -> Result<(), Box<dyn Error>> {
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2025, 2, 14).ok_or("Invalid date")?;
        let original = date.and_hms_opt(9, 20, 15).ok_or("Invalid time")?;
        let json = original.to_json();
        let context = JsJsonContext::new("test");

        let result = chrono::NaiveDateTime::from_json(context, json);
        assert!(result.is_ok());

        let deserialized = result?;
        assert_eq!(original, deserialized);

        Ok(())
    }

    #[test]
    fn naive_datetime_invalid_format() {
        use chrono::NaiveDateTime;

        let invalid_str = "2025-07-10 18:45:30"; // Wrong format (space-separated instead of ISO)
        let json = JsJson::String(invalid_str.to_string());
        let context = JsJsonContext::new("test");

        let result = NaiveDateTime::from_json(context, json);
        assert!(result.is_err());

        if let Err(err_context) = result {
            assert!(err_context.to_string().contains("DateTime parsing failed"));
        } else {
            panic!("Expected Err");
        }
    }
}
