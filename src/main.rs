#![allow(non_snake_case)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;

pub fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER:AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

struct BoxRefCell<T> {
    value: RefCell<T>,
}

impl<T> BoxRefCell<T> {
    fn new(value: T) -> BoxRefCell<T> {
        BoxRefCell {
            value: RefCell::new(value),
        }
    }

    fn get<R>(&self, getter: fn(&T) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        getter(&state)
    }

    fn change<D, R>(&self, data: D, changeFn: fn(&mut T, D) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        changeFn(&mut state, data)
    }
}

struct DependenciesInner {
    computed: HashMap<u64, ComputedRefresh>,          //To wykorzystujemy do wytrigerowania odpowiednich akcji
    client: HashMap<u64, ClientRefresh>,                //
    relations: HashMap<u64, u64>,           //relacje zaleności
    revertRelations: HashMap<u64, u64>,          //wykorzystywane do powiadamiania o konieczności przeliczenia
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            computed: HashMap::new(),
            client: HashMap::new(),
            relations: HashMap::new(),
            revertRelations: HashMap::new(),
        }
    }
}
struct Dependencies {
    inner: BoxRefCell<DependenciesInner>,
}

impl Dependencies {
    fn new() -> Rc<Dependencies> {
        Rc::new(
            Dependencies {
                inner: BoxRefCell::new(DependenciesInner::new())
            }
        )
    }

    fn newValue<T: Debug>(self: &Rc<Dependencies>, value: T) -> Rc<Value<T>> {
        Value::new(self.clone(), value)
    }

    fn triggerChange(self: &Rc<Dependencies>, id: u64) {

        //rozglaszamy po grafie nieswieze wartosci
        //wywolujemy ponowne przeliczenia

        todo!();
    }

    fn addRelation(self: &Rc<Dependencies>, parent: u64, target: ComputedRefresh) {

        self.inner.change((parent, target), |state, (parent, target)| {
            let targetId = target.getId();
            state.computed.insert(targetId, target);
        });

        todo!();
    }

    fn addRelationToClient(self: &Rc<Dependencies>, parent: u64, client: ClientRefresh) {

        todo!();
    }
}

struct ValueInner<T: 'static> {
    value: Rc<T>
}

impl<T: 'static> ValueInner<T> {
    fn new(value: T) -> ValueInner<T> {
        ValueInner {
            value: Rc::new(value)
        }
    }
}

struct Value<T: Debug + 'static> {
    id: u64,
    refCell: BoxRefCell<ValueInner<T>>,
    deps: Rc<Dependencies>,
}

impl<T: Debug + 'static> Value<T> {
    pub fn new(deps: Rc<Dependencies>, value: T) -> Rc<Value<T>> {
        Rc::new(Value {
            id: get_unique_id(),
            refCell: BoxRefCell::new(ValueInner::new(value)),
            deps
        })
    }

    pub fn setValue(self: &Rc<Value<T>>, value: T) /* -> Vec<Rc<Client>> */ {                          //TODO - trzeba odebrać i wywołać
        self.refCell.change(value, |state, value| {
            println!("nowa wartosc {:?}", value);
            state.value = Rc::new(value);
        });

        self.deps.triggerChange(self.id);
    }

    pub fn getValue(&self) -> Rc<T> {
        let value = self.refCell.get(|state| {
            state.value.clone()
        });

        value
    }

    pub fn toComputed(self: &Rc<Value<T>>) -> Rc<Computed<T>> {
        let selfClone = self.clone();

        let getValue = Box::new(move || {
            selfClone.getValue()
        });

        let computed = Computed::newRc(self.deps.clone(), getValue);

        self.deps.addRelation(self.id, computed.getComputedRefresh());

        computed
    }
}

// struct ComputedValue<T: 'static> {
//     value: Rc<T>,
// }

// impl<T: 'static> ComputedValue<T> {
//     pub fn new(value: Rc<T>) -> BoxRefCell<ComputedValue<T>> {
//         BoxRefCell::new(ComputedValue {
//             value,
//         })
//     }
// }

struct ComputedRefresh {
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

    fn setAsUnfreshInner(&self) {
        self.isFreshCell.change((), |state, _data| {
            *state = false;
        });
    }

    fn getId(&self) -> u64 {
        self.id
    }
}

struct Computed<T: Debug + 'static> {
    deps: Rc<Dependencies>,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
    id: u64,
    isFreshCell: Rc<BoxRefCell<bool>>,
    valueCell: BoxRefCell<Rc<T>>,
}

impl<T: Debug + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + 'static>(deps: Rc<Dependencies>, getValue: Box<F>) -> Rc<Computed<T>> {
        let newGetValue = Box::new(move || {
            Rc::new(getValue())
        });

        let value = newGetValue();

        Rc::new(
            Computed {
                deps,
                getValueFromParent: newGetValue,
                id: get_unique_id(),
                isFreshCell: Rc::new(BoxRefCell::new(true)),
                valueCell: BoxRefCell::new(value),
            }
        )
    }

    pub fn newRc<F: Fn() -> Rc<T> + 'static>(deps: Rc<Dependencies>, getValue: Box<F>) -> Rc<Computed<T>> {
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

    fn getComputedRefresh(&self) -> ComputedRefresh {
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

                println!("params {:?} {:?}", &aValue, &bValue);


                let result = calculate(aValue.as_ref(), bValue.as_ref());

                println!("result {:?}", result);
                result
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
            println!("wyliczam nowa wartosc ====> {:?}", result);
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
        let client = Client::new(self.clone(), call);

        self.deps.addRelationToClient(self.id, client.getClientRefresh());

        // let unsubscribe = self.addSubscription(Observer::fromClient(client.clone()));
        // client.setUnsubscribe(unsubscribe);

        client
    }
}

                                                        //TODO
// impl<T> Drop for Computed<T> {
//     fn drop(&mut self) {
//         println!("Rc<Computed<T>> ----> DROP");

//         todo!();

//         //Trzeba odsubskrybowac zrodla danych
//     }
// }


 
struct ClientRefresh {
    id: u64,
    refresh: Rc<BoxRefCell<Box<dyn Fn()>>>,
}

impl ClientRefresh {
    fn new(id: u64, refresh: Rc<BoxRefCell<Box<dyn Fn()>>>) -> ClientRefresh {
        ClientRefresh {
            id,
            refresh,
        }
    }

    fn recalculate(&self) {
        self.refresh.get(|state| {
            state();
        });
    }

    fn getId(&self) -> u64 {
        self.id
    }
}

struct Client {
    id: u64,
    refresh: Rc<BoxRefCell<Box<dyn Fn()>>>,
}

impl Client {
    fn new<T: Debug + 'static>(computed: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Client {
        let refresh = Box::new(move || {
            let value = computed.getValue();
            call(value);
        });
        
        refresh();

        Client {
            id: get_unique_id(),
            refresh: Rc::new(BoxRefCell::new(refresh))
        }
    }

    fn getClientRefresh(&self) -> ClientRefresh {
        ClientRefresh::new(self.id, self.refresh.clone())
    }

    fn off(self: Client) {
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("Client ----> DROP");

        todo!();

        //Trzeba odsubskrybowac zrodla danych
    }
}

fn main() {
    println!("Hello, world!");

    let root = Dependencies::new();

    let val1 = root.newValue(4);

    println!("ETAP 001");
    let val2 = root.newValue(5);

    println!("ETAP 002");

    let com1: Rc<Computed<i32>> = val1.toComputed();
    println!("ETAP 003");

    let com2: Rc<Computed<i32>> = val2.toComputed();

    println!("ETAP 004");


    let sum = Computed::from2(com1, com2, |a: &i32, b: &i32| -> i32 {
        println!("JESZCZE RAZ LICZE");
        a + b
    });

    println!("ETAP 005");

    let subscription = sum.subscribe(Box::new(|sum: Rc<i32>| {
        println!("___Suma: {}___", sum);
    }));

    println!("ETAP aaa");
    val1.setValue(333);

    println!("ETAP bbb");
    val2.setValue(888);

    println!("ETAP ccc");


    subscription.off();

    val2.setValue(889);
}



/*

            zarzadzanie subskrybcjami



            jesli liczba subskrybcji spadnie do zera, to wtedy trzeba wyczyscic subskrybcje

*/