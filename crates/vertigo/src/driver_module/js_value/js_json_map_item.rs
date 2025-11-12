use std::collections::BTreeMap;

use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonSerialize, from_json};

pub struct MapItem<K: JsJsonSerialize + JsJsonDeserialize, V: JsJsonSerialize + JsJsonDeserialize> {
    pub key: K,
    pub value: V,
}

impl<K: JsJsonSerialize + JsJsonDeserialize, V: JsJsonSerialize + JsJsonDeserialize> JsJsonSerialize for MapItem<K, V> {
    fn to_json(self) -> JsJson {
        let mut object = BTreeMap::<String, JsJson>::new();
        object.insert("k".into(), self.key.to_json());
        object.insert("v".into(), self.value.to_json());
        JsJson::Object(object)
    }
}


impl<K: JsJsonSerialize + JsJsonDeserialize, V: JsJsonSerialize + JsJsonDeserialize> JsJsonDeserialize for MapItem<K, V> {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let mut map = json.get_hashmap(&context)?;

        let k = map.remove("k");
        let v = map.remove("v");

        if map.is_empty() == false {
            return Err(context.add("Only the keys (k and v) were expected."));
        }

        if let (Some(k), Some(v)) = (k, v) {
            let k = from_json::<K>(k).map_err(|error| {
                context.add(format!("key error => {}", error))
            })?;

            let v = from_json::<V>(v).map_err(|error| {
                context.add(format!("value error => {}", error))
            })?;

            return Ok(MapItem {
                key: k,
                value: v
            });
        }

        return Err(context.add("Expected at this level of key k and v"));
    }
}
