use std::rc::Rc;

#[derive(Debug)]
struct JsJsonContextInner {
    parent: Option<Rc<JsJsonContextInner>>,
    current: String,
}

#[derive(Clone, Debug)]
pub struct JsJsonContext {
    inner: Rc<JsJsonContextInner>,
}

impl JsJsonContext {
    pub fn new(current: impl Into<String>) -> JsJsonContext {
        Self {
            inner: Rc::new(JsJsonContextInner {
                parent: None,
                current: current.into(),
            }),
        }
    }

    pub fn add(&self, child: impl ToString) -> JsJsonContext {
        Self {
            inner: Rc::new(JsJsonContextInner {
                parent: Some(self.inner.clone()),
                current: child.to_string(),
            }),
        }
    }

    pub fn convert_to_string(&self) -> String {
        let mut path = Vec::new();
        let mut current = self.inner.clone();

        loop {
            path.push(current.current.clone());

            let Some(parent) = current.parent.clone() else {
                return path.into_iter().rev().collect::<Vec<_>>().join(" -> ");
            };

            current = parent;
        }
    }
}

impl std::fmt::Display for JsJsonContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.convert_to_string())
    }
}

impl std::error::Error for JsJsonContext {}
