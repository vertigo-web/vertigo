use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    Dependencies::Dependencies,
    Client::Client,
    RefreshToken::RefreshToken,
    get_unique_id::get_unique_id,
};




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

        let id = get_unique_id();
        let isFreshCell = Rc::new(BoxRefCell::new(true));

        deps.startGetValueBlock();
        let value = getValue();
        deps.endGetValueBlock(
            RefreshToken::newComputed(id, isFreshCell.clone())
        );

        Computed {
            inner: Rc::new(ComputedInner {
                deps,
                getValueFromParent: getValue,
                id,
                isFreshCell: isFreshCell,
                valueCell: BoxRefCell::new(value),
            })
        }
    }

    pub fn getComputedRefresh(&self) -> RefreshToken {
        RefreshToken::newComputed(self.inner.id, self.inner.isFreshCell.clone())
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(&A, &B) -> T
    ) -> Computed<T> {
        let deps = a.inner.deps.clone();

        let getValue = Box::new(move || {
            let aValue = a.getValue();
            let bValue = b.getValue();

            let result = calculate(aValue.as_ref(), bValue.as_ref());

            Rc::new(result)
        });

        let result = Computed::new(deps, getValue);

        result
    }

    pub fn getValue(&self) -> Rc<T> {
        let inner = self.inner.as_ref();
        let selfId = inner.id;
        let deps = inner.deps.clone();

        deps.reportDependenceInStack(selfId);

        let shouldRecalculate = {
            self.inner.isFreshCell.changeNoParams(|state|{
                let shouldRecalculate = *state == false;
                *state = true;
                shouldRecalculate
            })
        };

        println!("**** computed getValue **** shouldRecalculate={}", shouldRecalculate);

        let newValue = if shouldRecalculate {
            deps.startGetValueBlock();

            let result = {
                let ComputedInner { getValueFromParent, .. } = self.inner.as_ref();
                getValueFromParent()
            };

            deps.endGetValueBlock(self.getComputedRefresh());
            Some(result)
        } else {
            None
        };

        let result = inner.valueCell.change(newValue, |state, newValue| {
            if let Some(value) = newValue {
                *state = value;
            }

            (*state).clone()
        });

        result
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        let client = Client::new(self.inner.deps.clone(), self.clone(), Box::new(call));

        self.inner.deps.addRelation(self.inner.id, client.getClientRefresh());

        client
    }

    pub fn map<K: Debug>(self, fun: fn(&T) -> K) -> Computed<K> {
        let deps = self.inner.deps.clone();

        let getValue = Box::new(move || {
            let value = self.getValue();
            let result = fun(value.as_ref());
            Rc::new(result)
        });

        let result = Computed::new(deps, getValue);
        result
    }
}

