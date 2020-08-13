use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Client::Client,
};


pub struct ComputedRefresh {
    id: u64,
    isFreshCell: Rc<BoxRefCell<bool>>,
}

impl ComputedRefresh {
    fn new(id: u64, isFreshCell: Rc<BoxRefCell<bool>>) -> ComputedRefresh {
        ComputedRefresh {
            id,
            isFreshCell,
        }
    }

    pub fn setAsUnfreshInner(&self) {
        self.isFreshCell.change((), |state, _data| {
            *state = false;
        });
    }

    pub fn getId(&self) -> u64 {
        self.id
    }
}

pub struct ComputedInner<T: Debug + 'static> {
    deps: Dependencies,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
    id: u64,
    isFreshCell: Rc<BoxRefCell<bool>>,
    valueCell: BoxRefCell<Rc<T>>,
}

impl<T: Debug> Drop for ComputedInner<T> {
    fn drop(&mut self) {
        println!("Computed<T> ----> DROP");
        self.deps.removeRelation(self.id);
    }
}


pub struct Computed<T: Debug + 'static> {
    inner: Rc<ComputedInner<T>>,
}

impl<T: Debug> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone()
        }
    }
}

impl<T: Debug + 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, getValue: Box<F>) -> Computed<T> {
        let value = getValue();
        Computed {
            inner: Rc::new(ComputedInner {
                deps,
                getValueFromParent: getValue,
                id: get_unique_id(),
                isFreshCell: Rc::new(BoxRefCell::new(true)),
                valueCell: BoxRefCell::new(value),
            })
        }
    }

    pub fn getComputedRefresh(&self) -> ComputedRefresh {
        ComputedRefresh::new(self.inner.id, self.inner.isFreshCell.clone())
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(&A, &B) -> T
    ) -> Computed<T> {

        let getValue = {
            let a = a.clone();
            let b = b.clone();

            Box::new(move || {
                let aValue = a.getValue();
                let bValue = b.getValue();

                let result = calculate(aValue.as_ref(), bValue.as_ref());

                Rc::new(result)
            })
        };

        let result = Computed::new(a.inner.deps.clone(), getValue);

        result.inner.deps.addRelation(a.inner.id, result.getComputedRefresh());
        result.inner.deps.addRelation(b.inner.id, result.getComputedRefresh());

        result
    }
    
    pub fn getValue(&self) -> Rc<T> {
        let inner = self.inner.as_ref();
        let ComputedInner { getValueFromParent, isFreshCell, valueCell, .. } = inner;

        let isFresh = isFreshCell.get(|state|{
            *state
        });

        println!("**** computed getValue **** {}", isFresh);

        let newValue = if isFresh == false {
            let result = getValueFromParent();
            Some(result)
        } else {
            None
        };

        let result = valueCell.change(newValue, |state, newValue| {
            if let Some(value) = newValue {
                *state = value;
            }

            (*state).clone()
        });

        result
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        let client = Client::new(self.inner.deps.clone(), self.clone(), Box::new(call));

        self.inner.deps.addRelationToClient(self.inner.id, client.getClientRefresh());

        client
    }

    pub fn map<K: Debug>(self, fun: fn(&T) -> K) -> Computed<K> {

        let getValue = {
            let selfClone = self.clone();
        

            Box::new(move || {
                let value = selfClone.getValue();

                let result = fun(value.as_ref());

                Rc::new(result)
            })
        };

        let result = Computed::new(self.inner.deps.clone(), getValue);

        result.inner.deps.addRelation(self.inner.id, result.getComputedRefresh());

        result
    }
}

