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

fn getId() -> u64 {
    get_unique_id()
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

trait ComputedTrait {
    fn setAsUnfresh(&self) -> Vec<Rc<Client>>;
}

enum Observer {
    Computed {
        id: u64,
        refVal: Box<dyn ComputedTrait>,         //Tutaj zwracamy listę z clientami do odświzenia
    },
    Client {
        id: u64,
        refVal: Rc<Client>,                     //Tutaj trigger nie wywołuje nic wgłąb, po prostu zwraca referencję
    }
}

impl Observer {
    fn getId(&self) -> u64 {
        match self {
            Observer::Computed { id, .. } => id.clone(),
            Observer::Client { id, .. } => id.clone()
        }
    }

    fn call(&self) -> Vec<Rc<Client>> {
        match self {
            Observer::Computed { refVal, .. } => {
                refVal.setAsUnfresh()
            },
            Observer::Client { refVal, .. } => {
                vec!(refVal.clone())
            }
        }
    }

    fn fromComputed<T: Debug>(computed: Rc<Computed<T>>) -> Observer {
        Observer::Computed {
            id: getId(),
            refVal: Box::new(computed)
        }
    }

    fn fromClient(client: Rc<Client>) -> Observer {
        Observer::Client {
            id: getId(),
            refVal: client,
        }
    }

    fn dump(&self) -> String {
        match self {
            Observer::Computed { .. } => {
                "Observer::Computed".into()
            },
            Observer::Client { .. } => {
                "Observer::Client".into()
            }
        }
    }
}

struct Unsubscribe {
    parent: Rc<Subscription>,
    id: u64,
}

impl Unsubscribe {
    fn new(parent: Rc<Subscription>, id: u64) -> Unsubscribe {
        Unsubscribe {
            parent,
            id
        }
    }
}

impl Drop for Unsubscribe {
    fn drop(&mut self) {
        let Unsubscribe { parent, id } = self;
        parent.remove(id);
    }
}

struct Subscription {
    list: BoxRefCell<HashMap<u64, Rc<Observer>>>,
}

impl Subscription {
    pub fn new() -> Rc<Subscription> {
        Rc::new(Subscription {
            list: BoxRefCell::new(HashMap::new())
        })
    }

    pub fn add(self: &Rc<Subscription>, observer: Observer) -> Unsubscribe {
        let observer = Rc::new(observer);

        println!("############ subskrybcja ############ {}", observer.dump());

        self.list.change(observer.clone(), |state, observer|{
            let id = observer.getId();
            let result = state.insert(id, observer);

            if result.is_some() {
                panic!("Coś poszło nie tak");
            }
        });

        let id = observer.getId();

        Unsubscribe::new(self.clone(), id)
    }

    pub fn trigger(&self) -> Vec<Rc<Client>> {
        let mut out: Vec<Rc<Client>> = Vec::new();

        let listToCall = self.list.get(|state|{
            let result: Vec<Rc<Observer>> = state.iter().map(|(_, value)| { value }).cloned().collect();
            result
        });

        for item in listToCall.iter() {
            let mut subList = item.call();
            out.append(&mut subList);
        }

        out
    }

    pub fn remove(self: &Rc<Subscription>, id: &u64) {
        self.list.change(id, |state, id|{
            let result = state.remove(&id);

            if result.is_none() {
                panic!("Błąd usuwania");
            }
        });

        // let mut list = self.list.borrow_mut();
        // let result = list.remove(&id);

        // if result.is_none() {
        //     panic!("Błąd usuwania");
        // }
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
    refCell: BoxRefCell<ValueInner<T>>,
    subscription: Rc<Subscription>,
}

impl<T: Debug + 'static> Value<T> {
    pub fn new(value: T) -> Rc<Value<T>> {
        Rc::new(Value {
            refCell: BoxRefCell::new(ValueInner::new(value)),
            subscription: Subscription::new(),
        })
    }

    pub fn setValue(self: &Rc<Value<T>>, value: T) /* -> Vec<Rc<Client>> */ {                          //TODO - trzeba odebrać i wywołać

        self.refCell.change(value, |state, value| {
            println!("nowa wartosc {:?}", value);
            state.value = Rc::new(value);
        });

        //todo!("Trzeba odebrac klientow do uruchomienia");

        let list = self.subscription.trigger();

        for item in list {
            item.recalculate();
        }
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

        let computed = Computed::newRc(getValue);

        let unsubscribe = self.subscription.add(Observer::fromComputed(computed.clone()));
        computed.addToUnsubscribeList(unsubscribe);

        computed
    }
}

struct ComputedValue<T: 'static> {
    isFresh: bool,
    value: Rc<T>,
    unsubscribeList: Vec<Unsubscribe>,
    subscription: Rc<Subscription>
}

impl<T: 'static> ComputedValue<T> {
    pub fn new(value: Rc<T>) -> BoxRefCell<ComputedValue<T>> {
        BoxRefCell::new(ComputedValue {
            isFresh: true,
            value,
            unsubscribeList: Vec::new(),
            subscription: Subscription::new(),
        })
    }
}

struct Computed<T: Debug + 'static> {
    refCell: BoxRefCell<ComputedValue<T>>,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
}

impl<T: Debug + 'static> Computed<T> {
    pub fn new<F: Fn() -> T + 'static>(getValue: Box<F>) -> Rc<Computed<T>> {
        let newGetValue = Box::new(move || {
            Rc::new(getValue())
        });

        let value = newGetValue();

        Rc::new(
            Computed {
                getValueFromParent: newGetValue,
                refCell: ComputedValue::new(value),
            }
        )
    }

    pub fn newRc<F: Fn() -> Rc<T> + 'static>(getValue: Box<F>) -> Rc<Computed<T>> {
        let value = getValue();
        Rc::new(
            Computed {
                getValueFromParent: getValue,
                refCell: ComputedValue::new(value),
            }
        )
    }

    fn addToUnsubscribeList(self: &Rc<Computed<T>>, unsubscribe: Unsubscribe) {
        self.refCell.change(unsubscribe, |state, unsubscribe| {
            state.unsubscribeList.push(unsubscribe);
        });
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Rc<Computed<A>>,
        b: Rc<Computed<B>>,
        calculate: fn(Rc<A>, Rc<B>) -> T
    ) -> Rc<Computed<T>> {

        let getValue = {
            let a = a.clone();
            let b = b.clone();

            Box::new(move || {
                let aValue = a.getValue();
                let bValue = b.getValue();

                println!("params {:?} {:?}", &aValue, &bValue);

                let result = calculate(aValue, bValue);

                println!("result {:?}", result);
                result
            })
        };

        let result = Computed::new(getValue);

        let aUnsubscribe = a.addSubscription(Observer::fromComputed(result.clone()));
        let bUnsubscribe = b.addSubscription(Observer::fromComputed(result.clone()));

        result.addToUnsubscribeList(aUnsubscribe);
        result.addToUnsubscribeList(bUnsubscribe);

        result
    }

    pub fn getValue(&self) -> Rc<T> {
        let Computed { getValueFromParent, refCell } = self;

        let isFresh = refCell.get(|state|{
            state.isFresh
        });

        println!("**** computed getValue **** {}", isFresh);

        let newValue = if isFresh == false {
            let result = getValueFromParent();
            println!("wyliczam nowa wartosc ====> {:?}", result);
            Some(result)
        } else {
            None
        };

        let result = refCell.change(newValue, |state, newValue| {
            if let Some(value) = newValue {
                state.value = value;
            }
            
            state.value.clone()
        });

        result
    }

    pub fn setAsUnfreshInner(&self) -> Vec<Rc<Client>> {
        let subscription = self.refCell.change(
            (),
            |state, _data| {
                println!("oznaczam jako nieswieze");
                state.isFresh = false;
                state.subscription.clone()
            }
        );

        subscription.trigger()
    }

    pub fn addSubscription(&self, observer: Observer) -> Unsubscribe {
        let unsubscribe = self.refCell.change(
            observer,
            |state, observer: Observer| {
                let unsubscribe = state.subscription.add(observer);
                unsubscribe
            }
        );

        unsubscribe
    }

    pub fn subscribe(self: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Rc<Client> {
        let client = Client::new(self.clone(), call);

        let unsubscribe = self.addSubscription(Observer::fromClient(client.clone()));
        client.setUnsubscribe(unsubscribe);

        client
    }
}

// impl<T> Drop for Computed<T> {
//     fn drop(&mut self) {
//         println!("Rc<Computed<T>> ----> DROP");
//     }
// }

impl<T: Debug + 'static> ComputedTrait for Rc<Computed<T>> {
    fn setAsUnfresh(&self) -> Vec<Rc<Client>> {
        self.setAsUnfreshInner()
    }
}

struct Client {
    refresh: Box<dyn Fn()>,
    _unsubscribe: BoxRefCell<Option<Unsubscribe>>,
}

impl Client {
    fn new<T: Debug + 'static>(computed: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Rc<Client> {
        let refresh = Box::new(move || {
            let value = computed.getValue();
            call(value);
        });
        
        refresh();

        Rc::new(
            Client {
                refresh,
                _unsubscribe: BoxRefCell::new(None),
            }
        )
    }

    fn setUnsubscribe(&self, unsubscribe: Unsubscribe) {
        self._unsubscribe.change(unsubscribe, |state: &mut Option<Unsubscribe>, unsubscribe: Unsubscribe| {
            if state.is_some() {
                panic!("Nic tu nie powinno być");
            }
        
            *state = Some(unsubscribe);
        });
    }

    fn off(self: Rc<Client>) {}

    fn recalculate(&self) {
        println!(" ..... recalculate start ..... ");
        let Client { refresh, .. } = self;
        refresh();
        println!(" ..... recalculate stop ..... ");
    }
}

fn main() {
    println!("Hello, world!");

    let val1 = Value::new(4);

    println!("ETAP 001");
    let val2 = Value::new(5);

    println!("ETAP 002");

    let com1: Rc<Computed<i32>> = val1.toComputed();
    println!("ETAP 003");

    let com2: Rc<Computed<i32>> = val2.toComputed();

    println!("ETAP 004");


    let sum = Computed::from2(com1, com2, |a: Rc<i32>, b: Rc<i32>| -> i32 {
        println!("JESZCZE RAZ LICZE");
        a.as_ref() + b.as_ref()
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
}
