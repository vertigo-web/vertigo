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
        }
        out.push(char);
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
            string_escape(value)
        );
    }

    pub fn build(self) -> String {
        let mut records: Vec<String> = Vec::new();

        for (key, value) in self.data.into_iter() {
            records.push(format!("\"{}\":\"{}\"", key, value));
        }

        let content = records.as_slice().join(":");

        format!("{{{}}}", content)
    }
}