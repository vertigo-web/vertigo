use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Client::Client,
    RefreshToken::RefreshToken,
};



pub struct ComputedBuilder {
    deps: Dependencies,
    id: u64,
    isFreshCell: Rc<BoxRefCell<bool>>,
}

impl ComputedBuilder {
    pub fn new(deps: Dependencies) -> ComputedBuilder {
        ComputedBuilder {
            deps,
            id: get_unique_id(),
            isFreshCell: Rc::new(BoxRefCell::new(true)),
        }
    }

    pub fn getComputedRefresh(&self) -> RefreshToken {
        RefreshToken::newComputed(self.id, self.isFreshCell.clone())
    }

    pub fn build<T: Debug, F: Fn() -> Rc<T> + 'static>(self, getValue: Box<F>) -> Computed<T> {
        let ComputedBuilder { deps, id, isFreshCell} = self;
        Computed::new(deps, id, isFreshCell, getValue)
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
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, id: u64,  isFreshCell: Rc<BoxRefCell<bool>>, getValue: Box<F>) -> Computed<T> {

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

    fn getComputedRefresh(&self) -> RefreshToken {
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

        let result = ComputedBuilder::new(deps).build(getValue);

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

        let result = ComputedBuilder::new(deps).build(getValue);
        result
    }
}

