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

pub struct Computed<T: Debug + 'static> {
    deps: Rc<Dependencies>,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
    id: u64,
    isFreshCell: Rc<BoxRefCell<bool>>,
    valueCell: BoxRefCell<Rc<T>>,
}

impl<T: Debug + 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Rc<Dependencies>, getValue: Box<F>) -> Rc<Computed<T>> {
        let value = getValue();
        Rc::new(
            Computed {
                deps,
                getValueFromParent: getValue,
                id: get_unique_id(),
                isFreshCell: Rc::new(BoxRefCell::new(true)),
                valueCell: BoxRefCell::new(value),
            }
        )
    }

    pub fn getComputedRefresh(&self) -> ComputedRefresh {
        ComputedRefresh::new(self.id, self.isFreshCell.clone())
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Rc<Computed<A>>,
        b: Rc<Computed<B>>,
        calculate: fn(&A, &B) -> T
    ) -> Rc<Computed<T>> {

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

        let result = Computed::new(a.deps.clone(), getValue);

        result.deps.addRelation(a.id, result.getComputedRefresh());
        result.deps.addRelation(b.id, result.getComputedRefresh());

        result
    }
    
    pub fn getValue(&self) -> Rc<T> {
        let Computed { getValueFromParent, isFreshCell, valueCell, .. } = self;

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

    pub fn subscribe(self: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Client {
        let client = Client::new(self.deps.clone(), self.clone(), call);

        self.deps.addRelationToClient(self.id, client.getClientRefresh());

        client
    }
}

impl<T: Debug> Drop for Computed<T> {
    fn drop(&mut self) {
        println!("Rc<Computed<T>> ----> DROP");
        self.deps.removeRelation(self.id);
    }
}

