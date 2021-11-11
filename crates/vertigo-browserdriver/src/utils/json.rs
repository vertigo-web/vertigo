use std::collections::HashMap;

pub struct JsonMapBuilder {
    data: HashMap<String, String>,
}

fn string_escape(text: &str) -> String {
    let new_capacity = 2 * text.len() + 2;
    let mut out: Vec<char> = Vec::with_capacity(new_capacity);

    for char in text.chars() {
        if char == '"' {
            out.push('\\');
            out.push(char);
        } else if char == '\n' {
            out.push('\\');
            out.push('n');
        } else {
            out.push(char);
        }
    }

    out.into_iter().collect()
}

impl JsonMapBuilder {
    pub fn new() -> JsonMapBuilder {
        JsonMapBuilder {
            data: HashMap::new()
        }
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.data.insert(
            string_escape(key),
            format!("\"{}\"", string_escape(value))
        );
    }

    pub fn set_u64(&mut self, key: &str, value: u64) {
        self.data.insert(
            string_escape(key),
            value.to_string()
        );
    }

    pub fn set_null(&mut self, key: &str) {
        self.data.insert(
            string_escape(key),
            "null".into()
        );
    }

    pub fn build(self) -> String {
        let mut records: Vec<String> = Vec::new();

        for (key, value) in self.data.into_iter() {
            records.push(format!("\"{}\":{}", key, value));
        }

        records.sort();

        let content = records.as_slice().join(",");

        format!("{{{}}}", content)
    }
}

//JSON Formatter & Validator
//https://jsonformatter.curiousconcept.com/

#[test]
fn basic() {
    let builder = JsonMapBuilder::new();
    let result = builder.build();
    assert_eq!(result, "{}");
}

#[test]
fn basic2() {
    let mut builder = JsonMapBuilder::new();
    builder.set_string("Content-Type", "application/json");
    let result = builder.build();
    assert_eq!(result, "{\"Content-Type\":\"application/json\"}");
}

#[test]
fn basic3() {
    let mut builder = JsonMapBuilder::new();
    builder.set_string("Content-Type", "application/json");
    builder.set_string("token", "3333");
    let result = builder.build();
    assert_eq!(result, "{\"Content-Type\":\"application/json\",\"token\":\"3333\"}");
}

#[test]
fn basic4() {
    let mut builder = JsonMapBuilder::new();
    builder.set_string("token", "33\"33");
    let result = builder.build();
    assert_eq!(result, "{\"token\":\"33\\\"33\"}");
}


#[test]
fn basic5() {
    let mut builder = JsonMapBuilder::new();
    builder.set_u64("token", 4);
    let result = builder.build();
    assert_eq!(result, "{\"token\":4}");
}

#[test]
fn basic6() {
    let mut builder = JsonMapBuilder::new();
    builder.set_null("token");
    let result = builder.build();
    assert_eq!(result, "{\"token\":null}");
}

#[test]
fn basic7() {
    let mut builder = JsonMapBuilder::new();
    builder.set_string("css", r#"color: crimson;
text-decoration: line-through;"#);
    let result = builder.build();
    assert_eq!(result, "{\"css\":\"color: crimson;\\ntext-decoration: line-through;\"}");
}
