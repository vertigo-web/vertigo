use crate::JsJson;

/// API access mock of vertigo::Driver
///
/// Mocks DOM access methods on vertigo::Driver used by `js!` macro
/// and allows to compare against expected result.
pub struct ApiMock(Vec<String>);

pub fn get_driver() -> ApiMock {
    ApiMock(vec!["vertigo::get_driver()".to_string()])
}

impl ApiMock {
    // Mocks:

    pub fn dom_access(mut self) -> Self {
        self.0.push(".api_access()".to_string());
        self
    }

    pub fn root(mut self, name: &str) -> Self {
        self.0.push(format!(".root(\"{name}\")"));
        self
    }

    pub fn get(mut self, name: &str) -> Self {
        self.0.push(format!(".get(\"{name}\")"));
        self
    }

    pub fn call(mut self, func: &str, args: Vec<JsJson>) -> Self {
        self.0.push(format!(".call(\"{func}\", {args:?})"));
        self
    }

    pub fn fetch(mut self) -> Self {
        self.0.push(".fetch()".to_string());
        self
    }

    // Utils:

    pub fn new_ref(id: i32) -> ApiMock {
        ApiMock(vec![format!("node_ref({id})")])
    }

    pub fn result(&self) -> String {
        self.0.join("\n")
    }

    pub fn expect(&self, expected: &str) {
        let expected = remove_indentation(expected);

        assert_eq!(self.result(), expected);
    }
}

fn remove_indentation(code: &str) -> String {
    code.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None // Remove empty lines
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
