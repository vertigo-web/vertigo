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

impl Clone for ComputedRefresh {
    fn clone(&self) -> Self {
        ComputedRefresh {
            id: self.id,
            isFreshCell: self.isFreshCell.clone(),
        }
    }
}



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

    pub fn getComputedRefresh(&self) -> ComputedRefresh {
        ComputedRefresh::new(self.id, self.isFreshCell.clone())
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

pub struct Getter<T: Debug + 'static> {
    isGet: bool,
    computed: Computed<T>,
}

impl<T: Debug + 'static> Getter<T> {
    fn new(computed: Computed<T>) -> Getter<T> {
        Getter {
            isGet: false,
            computed
        }
    }

    fn getValue(&mut self) -> Rc<T> {
        let value = self.computed.getValue();
        self.isGet = true;
        value
    }
}

impl<T: Debug + 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, id: u64,  isFreshCell: Rc<BoxRefCell<bool>>, getValue: Box<F>) -> Computed<T> {
        let value = getValue();
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

    pub fn from2<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(&A, &B) -> T
    ) -> Computed<T> {

        let deps = a.inner.deps.clone();
        let aId = a.inner.id;
        let bId = b.inner.id;
        let builder = ComputedBuilder::new(deps);
        let refresh = builder.getComputedRefresh();

        let getValue = {

            Box::new(move || {
                let aValue = a.getValue();
                let bValue = b.getValue();

                let result = calculate(aValue.as_ref(), bValue.as_ref());

                Rc::new(result)
            })
        };

        let result = builder.build(getValue);

        result.inner.deps.addRelation(aId, refresh.clone());
        result.inner.deps.addRelation(bId, refresh);

        result
    }

    pub fn from2Dyn<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(&mut Getter<A>, &mut Getter<B>) -> T
    ) -> Computed<T> {

        let deps = a.inner.deps.clone();
        let aId = a.inner.id;
        let bId = b.inner.id;
        let builder = ComputedBuilder::new(deps.clone());
        let refresh = builder.getComputedRefresh();

        let clientId = builder.id;

        let getValue = {

            Box::new(move || {
                let mut getterA = Getter::new(a.clone());
                let mut getterB = Getter::new(b.clone());
                
                deps.removeRelation(clientId);

                let result = calculate(&mut getterA, &mut getterB);

                if getterA.isGet {
                    deps.addRelation(aId, refresh.clone());
                }

                if getterB.isGet {
                    deps.addRelation(bId, refresh.clone());
                }
                
                Rc::new(result)
            })
        };

        builder.build(getValue)
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

        let deps = self.inner.deps.clone();
        let builder = ComputedBuilder::new(deps);
        let parentId = self.inner.id;
        let refresh = builder.getComputedRefresh();

        let getValue = {
            let selfClone = self.clone();
        

            Box::new(move || {
                let value = selfClone.getValue();

                let result = fun(value.as_ref());

                Rc::new(result)
            })
        };

        let result = builder.build(getValue);

        result.inner.deps.addRelation(parentId, refresh);

        result
    }
}

