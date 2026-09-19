use super::js_json_struct::JsJson;

/// The string-keyed map behind [`JsJson::Object`].
///
/// A sorted `Vec` rather than a `BTreeMap`, for size. The objects that cross the JS
/// boundary have a handful of keys each, so the tree never pays off at runtime - and
/// `BTreeMap<String, JsJson>`'s node machinery (insert, remove, the balancing, the
/// clone and drop descents) measured 11KB in a minimal app, which every application
/// links whether or not it ever builds an object of its own.
///
/// Sorted, because iteration order is the wire order: it has to stay the one `BTreeMap`
/// produced, or SSR stops being byte-reproducible.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct JsObject {
    entries: Vec<(String, JsJson)>,
}

impl JsObject {
    pub fn new() -> Self {
        JsObject {
            entries: Vec::new(),
        }
    }

    fn find(&self, key: &str) -> Result<usize, usize> {
        self.entries
            .binary_search_by(|(entry_key, _)| entry_key.as_str().cmp(key))
    }

    pub fn insert(&mut self, key: String, value: JsJson) -> Option<JsJson> {
        match self.find(&key) {
            Ok(index) => Some(std::mem::replace(&mut self.entries[index].1, value)),
            Err(index) => {
                self.entries.insert(index, (key, value));
                None
            }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<JsJson> {
        match self.find(key) {
            Ok(index) => Some(self.entries.remove(index).1),
            Err(_) => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsJson> {
        match self.find(key) {
            Ok(index) => Some(&self.entries[index].1),
            Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsJson> {
        match self.find(key) {
            Ok(index) => Some(&mut self.entries[index].1),
            Err(_) => None,
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.find(key).is_ok()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &JsJson> {
        self.entries.iter().map(|(_, value)| value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &JsJson)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }
}

impl IntoIterator for JsObject {
    type Item = (String, JsJson);
    type IntoIter = std::vec::IntoIter<(String, JsJson)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a JsObject {
    type Item = (&'a String, &'a JsJson);
    type IntoIter = Box<dyn Iterator<Item = (&'a String, &'a JsJson)> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl FromIterator<(String, JsJson)> for JsObject {
    fn from_iter<I: IntoIterator<Item = (String, JsJson)>>(iter: I) -> Self {
        let mut object = JsObject::new();
        for (key, value) in iter {
            object.insert(key, value);
        }
        object
    }
}

impl<const N: usize> From<[(String, JsJson); N]> for JsObject {
    fn from(entries: [(String, JsJson); N]) -> Self {
        entries.into_iter().collect()
    }
}
