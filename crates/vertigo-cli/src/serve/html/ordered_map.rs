use std::collections::HashMap;

#[derive(Clone)]
pub struct OrderedMap {
    order: Vec<String>,
    data: HashMap<String, String>,
}

impl OrderedMap {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();

        self.data.insert(name.clone(), value);

        if self.data.contains_key(&name) {
            let mut order = self
                .order
                .iter()
                .filter(|&item_name| item_name != &name)
                .cloned()
                .collect::<Vec<_>>();

            order.push(name);

            self.order = order;
        } else {
            self.order.push(name);
        }
    }

    pub fn get_iter(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        for key in self.order.iter() {
            let value = self.data.get(key).unwrap();
            result.push((key.clone(), value.clone()));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedMap;

    #[test]
    fn basic() {
        let mut list = OrderedMap::new();
        list.set("aa_k", "aa_v");
        list.set("bb_k", "bb_v");
        list.set("cc_k", "cc_v");

        assert_eq!(
            list.get_iter(),
            vec!(
                ("aa_k".to_string(), "aa_v".to_string()),
                ("bb_k".to_string(), "bb_v".to_string()),
                ("cc_k".to_string(), "cc_v".to_string()),
            )
        );

        let mut list = OrderedMap::new();
        list.set("aa_k", "aa_v");
        list.set("bb_k", "bb_v");
        list.set("cc_k", "cc_v");
        list.set("bb_k", "bbbbb_v");
        assert_eq!(
            list.get_iter(),
            vec!(
                ("aa_k".to_string(), "aa_v".to_string()),
                ("cc_k".to_string(), "cc_v".to_string()),
                ("bb_k".to_string(), "bbbbb_v".to_string()),
            )
        );
    }
}
