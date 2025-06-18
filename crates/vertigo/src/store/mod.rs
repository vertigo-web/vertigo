use crate::{transaction, Value};

pub trait Identifiable {
    type Key: PartialEq;

    fn get_id(&self) -> Self::Key;

    fn equivalent_to(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

pub trait Store<T> {

    fn set(&mut self, new: T);
}

impl<T, K> Store<Vec<T>> for Vec<Value<T>>
where
    T: Identifiable<Key = K> + PartialEq + Clone + 'static,
{
    fn set(&mut self, incoming: Vec<T>)
    {
        transaction(move |ctx| {
            let mut needs_reset = false;
            let mut new_array = vec![];

            // Search for anything deleted in incoming
            for old in self.iter() {
                if !incoming.iter().any(|v| v.equivalent_to(&old.get(ctx))) {
                    // Just mark that we need new array
                    needs_reset = true;
                    // No need to search for more
                    break;
                }
            }

            // In general, take all from incoming array
            for item in incoming {
                // If item was already on the list, update it and clone info new array
                if let Some(v) = self.iter().find(|v| v.get(ctx).equivalent_to(&item)) {
                    v.set(item);
                    new_array.push(v.clone());
                } else {
                    // If not, create new array, add to the new list and mark that we're replacing the list
                    new_array.push(Value::new(item));
                    needs_reset = true;
                }
            }

            if needs_reset {
                *self = new_array;
            }
        })
    }
}

impl Identifiable for i32 {
    type Key = Self;
    fn get_id(&self) -> Self::Key {
        *self
    }
}

#[cfg(test)]
mod tests {
    use crate::{transaction, Value};

    use super::Store;

    #[test]
    fn store_value() {
        let x = Value::new(1);
        x.set(2);
        transaction(|ctx| {
            assert_eq!(x.get(ctx), 2);
        })
    }

    #[test]
    fn store_vec() {
        let mut x = vec![Value::new(1)];
        let new = vec![1, 2];
        x.set(new.clone());
        transaction(|ctx| {
            assert_eq!(x[0].get(ctx), new[0]);
            assert_eq!(x[1].get(ctx), new[1]);
        })
    }
}
