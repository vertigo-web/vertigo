use rust_decimal::prelude::ToPrimitive;

use super::{
    JsJsonContext, JsJsonDeserialize, JsJsonSerialize,
    js_json_struct::{JsJson, JsJsonNumber},
};

impl JsJsonSerialize for rust_decimal::Decimal {
    fn to_json(self) -> JsJson {
        let val = self.to_f64().unwrap_or(0.0);
        JsJson::Number(JsJsonNumber(val))
    }
}

impl JsJsonDeserialize for rust_decimal::Decimal {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::Number(JsJsonNumber(val)) => {
                rust_decimal::Decimal::try_from(val).map_err(|err| {
                    let message = ["Decimal parsing failed: ", &err.to_string()].concat();
                    context.add(message)
                })
            }
            other => {
                let message = ["number expected, received ", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, JsJsonSerialize};
    use rust_decimal::Decimal;
    use std::{error::Error, str::FromStr};

    #[test]
    fn decimal_serialization() -> Result<(), Box<dyn Error>> {
        let dec = Decimal::from_str("123.45").map_err(|e| e.to_string())?;
        let json = dec.to_json();
        if let JsJson::Number(JsJsonNumber(val)) = json {
            assert_eq!(val, 123.45);
        } else {
            panic!("Expected JsJson::Number");
        }

        Ok(())
    }

    #[test]
    fn decimal_deserialization() -> Result<(), Box<dyn Error>> {
        let json = JsJson::Number(JsJsonNumber(123.45));
        let context = JsJsonContext::new("test");

        let result = Decimal::from_json(context, json)?;
        assert_eq!(
            result,
            Decimal::from_str("123.45").map_err(|e| e.to_string())?
        );

        Ok(())
    }

    #[test]
    fn decimal_round_trip() -> Result<(), Box<dyn Error>> {
        let original = Decimal::from_str("9876.5432").map_err(|e| e.to_string())?;
        let json = original.to_json();
        let context = JsJsonContext::new("test");

        let result = Decimal::from_json(context, json)?;
        assert_eq!(original, result);

        Ok(())
    }

    #[test]
    fn decimal_precision_loss() -> Result<(), Box<dyn Error>> {
        // 9007199254740993 is 2^53 + 1, which cannot be represented exactly in f64
        // It rounds to 9007199254740992 (2^53) during the to_f64() conversion
        let original = Decimal::from_str("9007199254740993").map_err(|e| e.to_string())?;
        let json = original.to_json();
        let context = JsJsonContext::new("test");

        let result = Decimal::from_json(context, json)?;
        assert_eq!(
            result,
            Decimal::from_str("9007199254740992").map_err(|e| e.to_string())?
        );

        Ok(())
    }
}
