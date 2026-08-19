use super::inner_value::InnerValue;

pub struct VecMut<V> {
    data: InnerValue<Vec<V>>,
}

impl<V> Default for VecMut<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> VecMut<V> {
    pub fn new() -> VecMut<V> {
        VecMut {
            data: InnerValue::new(Vec::new()),
        }
    }

    pub fn push(&self, value: V) {
        let state = self.data.get_mut();
        state.push(value);
    }

    pub fn take(&self) -> Vec<V> {
        let state = self.data.get_mut();
        std::mem::take(state)
    }

    pub fn for_each(&self, callback: impl Fn(&V)) {
        let state = self.data.get();
        for item in state.iter() {
            callback(item);
        }
    }
    pub fn map<K>(&self, map: impl Fn(&Vec<V>) -> K) -> K {
        let data = self.data.get_mut();
        map(&*data)
    }

    pub fn into_inner(self) -> Vec<V> {
        self.data.into_inner()
    }
}
