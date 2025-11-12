use vertigo::{JsJson, JsValue};

#[cfg(test)]
mod tests {
    use vertigo::JsValue;

    #[derive(PartialEq)]
    pub struct JsList(Vec<JsValue>);

    impl JsList {
        pub fn new() -> Self {
            Self(Vec::new())
        }

        pub fn add(mut self, value: JsValue) -> Self {
            self.0.push(value);
            self
        }

        pub fn str(mut self, value: impl Into<String>) -> Self {
            self.0.push(JsValue::str(value));
            self
        }

        pub fn u64(mut self, value: u64) -> Self {
            self.0.push(JsValue::U64(value));
            self
        }

        pub fn list(self, list: &[&str]) -> Self {
            let mut result = JsList::new();

            for item in list {
                result = result.add(JsValue::str(*item));
            }

            self.add(result.convert_to_value())
        }

        pub fn convert_to_value(self) -> JsValue {
            let Self(list) = self;
            JsValue::List(list)
        }
    }
}

pub struct Match<'a> {
    list: &'a [JsValue],
}

impl<'a> Match<'a> {
    pub fn new(value: &'a JsValue) -> Result<Match<'a>, ()> {
        if let JsValue::List(list) = value {
            return Ok(Match {
                list: list.as_slice(),
            });
        }

        Err(())
    }

    pub fn from_slice(list: &'a [JsValue]) -> Match<'a> {
        Match { list }
    }

    pub fn str(&self, pattern: &str) -> Result<Self, ()> {
        let list = self.list;

        let Some((JsValue::String(value), rest)) = list.split_first() else {
            return Err(());
        };

        if pattern == value {
            return Ok(Self { list: rest });
        }

        Err(())
    }

    pub fn u32(&self) -> Result<(Self, u32), ()> {
        let list = self.list;

        let Some((JsValue::U32(value), rest)) = list.split_first() else {
            return Err(());
        };

        Ok((Self { list: rest }, *value))
    }

    pub fn u64(&self) -> Result<(Self, u64), ()> {
        let list = self.list;

        let Some((JsValue::U64(value), rest)) = list.split_first() else {
            return Err(());
        };

        Ok((Self { list: rest }, *value))
    }

    pub fn string(&self) -> Result<(Self, String), ()> {
        let list = self.list;

        let Some((JsValue::String(value), rest)) = list.split_first() else {
            return Err(());
        };

        Ok((Self { list: rest }, value.clone()))
    }

    pub fn json(&self) -> Result<(Self, JsJson), ()> {
        let list = self.list;

        let Some((JsValue::Json(value), rest)) = list.split_first() else {
            return Err(());
        };

        Ok((Self { list: rest }, value.clone()))
    }

    #[allow(dead_code)]
    pub fn get_any(&self) -> Result<(Self, JsValue), ()> {
        let list = self.list;

        let Some((value, rest)) = list.split_first() else {
            return Err(());
        };

        Ok((Self { list: rest }, value.clone()))
    }

    #[allow(dead_code)]
    pub fn option_string(&self) -> Result<(Self, Option<String>), ()> {
        let list = self.list;

        let Some((first, rest)) = list.split_first() else {
            return Err(());
        };

        if let JsValue::String(value) = first {
            return Ok((Self { list: rest }, Some(value.clone())));
        }

        if let JsValue::Null = first {
            return Ok((Self { list: rest }, None));
        }

        Err(())
    }

    pub fn test_list(&self, list_pattern: &[&str]) -> Result<Self, ()> {
        let Match { list: inner_list } = self;

        let Some((JsValue::List(first_inner_list), rest)) = inner_list.split_first() else {
            return Err(());
        };

        if list_pattern.len() != first_inner_list.len() {
            return Err(());
        }

        for (index, item_pattern) in list_pattern.iter().enumerate() {
            let Some(value) = first_inner_list.get(index) else {
                return Err(());
            };

            let JsValue::String(inner_str) = value else {
                return Err(());
            };

            if *item_pattern != inner_str {
                return Err(());
            }
        }

        Ok(Self { list: rest })
    }

    pub fn test_list_with_fn<K>(
        &self,
        test_fn: impl Fn(Match<'a>) -> Result<K, ()>,
    ) -> Result<(Self, K), ()> {
        let Match { list: inner_list } = self;

        let Some((JsValue::List(first_inner_list), rest)) = inner_list.split_first() else {
            return Err(());
        };

        let new_self = Self { list: rest };

        if let Ok(result_test) = test_fn(Match::from_slice(first_inner_list.as_slice())) {
            return Ok((new_self, result_test));
        }

        Err(())
    }

    pub fn end(&self) -> Result<(), ()> {
        let Match { list: value } = self;

        if value.is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        println!("{:#?}", self.list);
        todo!()
    }
}

#[test]
fn basic() {
    use tests::JsList;

    fn match_hashrouter(arg: &JsValue) -> Result<(), ()> {
        let matcher = Match::new(arg)?;
        let matcher = matcher.test_list(&["api"])?;
        let matcher = matcher.test_list(&["get", "hashRouter"])?;
        let matcher = matcher.test_list(&["call", "get"])?;
        matcher.end()?;
        Ok(())
    }

    let pattern_get_hashrouter = JsList::new()
        .list(&["api"])
        .list(&["get", "hashRouter"])
        .list(&["call", "get"])
        .convert_to_value();

    assert_eq!(match_hashrouter(&pattern_get_hashrouter), Ok(()));

    let pattern_get_hashrouter = JsList::new()
        .list(&["api"])
        .list(&["get", "hashRouter"])
        .list(&["call", "get", "dddd"])
        .convert_to_value();

    assert_eq!(match_hashrouter(&pattern_get_hashrouter), Err(()));

    let pattern_get_hashrouter = JsList::new()
        .list(&["api"])
        .list(&["get", "hashRouter"])
        .convert_to_value();

    assert_eq!(match_hashrouter(&pattern_get_hashrouter), Err(()));

    let pattern_get_hashrouter = JsList::new()
        .list(&["api"])
        .list(&["get", "hashRouter"])
        .list(&["call", "get"])
        .list(&["aaaa"])
        .convert_to_value();

    assert_eq!(match_hashrouter(&pattern_get_hashrouter), Err(()));
}

#[test]
fn test_param() {
    use tests::JsList;

    let pattern_add_hashrouter = JsList::new()
        .list(&["api"])
        .list(&["get", "hashRouter"])
        .add(
            JsList::new()
                .str("call")
                .str("add")
                .u64(1)
                .convert_to_value(),
        )
        .convert_to_value();

    fn match_call_add(matcher: Match) -> Result<u64, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("add")?;
        let (matcher, callback_id) = matcher.u64()?;
        matcher.end()?;

        Ok(callback_id)
    }

    fn match_hashrouter_add(arg: &JsValue) -> Result<u64, ()> {
        let matcher = Match::new(arg)?;
        let matcher = matcher.test_list(&["api"])?;
        let matcher = matcher.test_list(&["get", "hashRouter"])?;
        let (matcher, callback_id) = matcher.test_list_with_fn(match_call_add)?;
        matcher.end()?;

        Ok(callback_id)
    }

    assert_eq!(match_hashrouter_add(&pattern_add_hashrouter), Ok(1));
}
