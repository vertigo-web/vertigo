use std::collections::HashMap;
use std::rc::Rc;

use super::js_json_struct::JsJson;

struct JsJsonContextInner {
    parent: Option<Rc<JsJsonContextInner>>,
    current: String,
}

#[derive(Clone)]
pub struct JsJsonContext {
    inner: Rc<JsJsonContextInner>,
}

impl JsJsonContext {
    pub fn new(current: impl Into<String>) -> JsJsonContext {
        Self {
            inner: Rc::new(JsJsonContextInner {
                parent: None,
                current: current.into()
            })
        }
    }

    pub fn add(&self, child: impl ToString) -> JsJsonContext {
        Self {
            inner: Rc::new(JsJsonContextInner {
                parent: Some(self.inner.clone()),
                current: child.to_string()
            })
        }
    }

    pub fn convert_to_string(&self) -> String {
        let mut path = Vec::new();
        let mut current = self.inner.clone();

        loop {
            path.push(current.current.clone());

            let Some(parent) = current.parent.clone() else {
                return path
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" -> ");
            };

            current = parent;
        }
    }
}

pub trait JsJsonSerialize {
    fn to_json(self) -> JsJson;
}

pub trait JsJsonDeserialize where Self: Sized {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext>;
}


impl JsJsonSerialize for String {
    fn to_json(self) -> JsJson {
        JsJson::String(self)
    }
}

impl JsJsonDeserialize for String {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::String(value) => Ok(value),
            other => {
                let message = ["string expected, received", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

impl JsJsonSerialize for u64 {
    fn to_json(self) -> JsJson {
        JsJson::Number(self as f64)
    }
}

impl JsJsonDeserialize for u64 {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::Number(value) => Ok(value as u64),
            other => {
                let message = ["number(u64) expected, received", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

impl JsJsonSerialize for i64 {
    fn to_json(self) -> JsJson {
        JsJson::Number(self as f64)
    }
}

impl JsJsonDeserialize for i64 {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::Number(value) => Ok(value as i64),
            other => {
                let message = ["number(i64) expected, received", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

impl JsJsonSerialize for u32 {
    fn to_json(self) -> JsJson {
        JsJson::Number(self as f64)
    }
}

impl JsJsonDeserialize for u32 {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::Number(value) => Ok(value as u32),
            other => {
                let message = ["number(u32) expected, received", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

impl JsJsonSerialize for i32 {
    fn to_json(self) -> JsJson {
        JsJson::Number(self as f64)
    }
}

impl JsJsonDeserialize for i32 {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        match json {
            JsJson::Number(value) => Ok(value as i32),
            other => {
                let message = ["number(i32) expected, received", other.typename()].concat();
                Err(context.add(message))
            }
        }
    }
}

impl<T: JsJsonSerialize> JsJsonSerialize for Vec<T> {
    fn to_json(self) -> JsJson {
        let mut list = Vec::new();

        for item in self {
            list.push(item.to_json());
        }

        JsJson::List(list)
    }
}

impl<T: JsJsonDeserialize> JsJsonDeserialize for Vec<T> {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let mut list = Vec::new();

        let JsJson::List(inner) = json else {
            let message = ["List expected, received", json.typename()].concat();
            return Err(context.add(message));
        };

        for (index, item) in inner.into_iter().enumerate() {
            list.push(T::from_json(context.add(index), item)?);
        }

        Ok(list)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Post {
    name: String,
    age: u64,
}

impl JsJsonSerialize for Post {
    fn to_json(self) -> JsJson {
        JsJson::Object(HashMap::from([
            ("name".to_string(), self.name.to_json()),
            ("age".to_string(), self.age.to_json()),
        ]))
    }
}

impl JsJsonDeserialize for Post {
    fn from_json(context: JsJsonContext, mut json: JsJson) -> Result<Self, JsJsonContext> {
        Ok(Self {
            name: json.get_property(&context, "name")?,
            age: json.get_property(&context, "age")?,
        })
    }
}

pub fn from_json<T: JsJsonDeserialize>(json: JsJson) -> Result<T, String> {
    let context = JsJsonContext::new("root");
    let result = T::from_json(context, json);
    result.map_err(|context| context.convert_to_string())
}

pub fn to_json<T: JsJsonSerialize>(value: T) -> JsJson {
    value.to_json()
}


#[test]
fn aaaa() {
    let aaa = JsJson::String("aaa".into());
    let aaa_post = from_json::<Post>(aaa);
    assert_eq!(aaa_post, Err(String::from("root -> object expected, received string")));

    let bbb = Post {
        name: "dsada".into(),
        age: 33
    };

    let ccc = bbb.clone().to_json();
    let ddd = from_json::<Post>(ccc).unwrap();
    assert_eq!(bbb, ddd);
}

#[test]
fn test_vec() {

    let aaa = Post {
        name: "aaa".into(),
        age: 11
    };

    let bbb = Post {
        name: "bbb".into(),
        age: 22
    };

    let ccc = vec!(aaa, bbb);

    let ddd = ccc.clone().to_json();

    let eee = from_json::<Vec<Post>>(ddd).unwrap();

    assert_eq!(ccc, eee);
}