#![allow(non_snake_case)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER:AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn getId() -> u64 {
    get_unique_id()
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

    fn fromComputed<T>(computed: Rc<Computed<T>>) -> Observer {
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
    list: RefCell<HashMap<u64, Observer>>,
}

impl Subscription {
    pub fn new() -> Rc<Subscription> {
        Rc::new(Subscription {
            list: RefCell::new(HashMap::new())
        })
    }

    pub fn add(self: &Rc<Subscription>, observer: Observer) -> Unsubscribe {
        let id = observer.getId();
        let mut list = self.list.borrow_mut();
        let result = list.insert(id, observer);

        if result.is_some() {
            panic!("Coś poszło nie tak");
        }

        Unsubscribe::new(self.clone(), id)
    }

    pub fn trigger(&self) -> Vec<Rc<Client>> {
        let mut out: Vec<Rc<Client>> = Vec::new();
        let mut list = self.list.borrow();
        for (_, item) in list.iter() {
            let mut subList = item.call();
            out.append(&mut subList);
        }

        out
    }

    pub fn remove(self: &Rc<Subscription>, id: &u64) {
        let mut list = self.list.borrow_mut();
        let result = list.remove(&id);

        if result.is_none() {
            panic!("Błąd usuwania");
        }
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

struct Value<T: 'static> {
    refCell: RefCell<ValueInner<T>>,
    subscription: Rc<Subscription>,
}

impl<T: 'static> Value<T> {
    pub fn new(value: T) -> Rc<Value<T>> {
        Rc::new(Value {
            refCell: RefCell::new(ValueInner::new(value)),
            subscription: Subscription::new(),
        })
    }

    pub fn setValue(self: &Rc<Value<T>>, value: T) -> Vec<Rc<Client>> {                          //TODO - trzeba odebrać i wywołać
        let mut inner = self.refCell.borrow_mut();
        inner.value = Rc::new(value);

        todo!("Trzeba odebrac klientow do uruchomienia");

        let list = self.subscription.trigger();

        for item in list {
            item.recalculate();
        }
    }

    pub fn getValue(&self) -> Rc<T> {
        let inner = self.refCell.borrow();
        (*inner).value.clone()
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
    pub fn new(value: Rc<T>) -> RefCell<ComputedValue<T>> {
        RefCell::new(ComputedValue {
            isFresh: true,
            value,
            unsubscribeList: Vec::new(),
            subscription: Subscription::new(),
        })
    }
}

struct Computed<T: 'static> {
    refCell: RefCell<ComputedValue<T>>,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
}

impl<T: 'static> Computed<T> {
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
        let mut inner = self.refCell.borrow_mut();
        inner.unsubscribeList.push(unsubscribe);
    }

    pub fn from2<A, B>(
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

                calculate(aValue, bValue)
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

        let mut inner = refCell.borrow();

        if inner.isFresh == false {
            inner.value = getValueFromParent();
            inner.isFresh = true;
        }

        inner.value.clone()
    }

    pub fn setAsUnfreshInner(&self) -> Vec<Rc<Client>> {
        let mut inner = self.refCell.borrow_mut();
        inner.isFresh = false;
        inner.subscription.trigger()
    }

    pub fn addSubscription(&self, observer: Observer) -> Unsubscribe {
        let inner = self.refCell.borrow_mut();
        let unsubscribe = inner.subscription.add(observer);
        unsubscribe
    }

    pub fn subscribe(self: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Rc<Client> {
        let client = Client::new(self.clone(), call);

        let unsubscribe = self.addSubscription(Observer::fromClient(client.clone()));

        client.setUnsubscribe(unsubscribe);

        client
    }
}
impl<T: 'static> ComputedTrait for Rc<Computed<T>> {
    fn setAsUnfresh(&self) -> Vec<Rc<Client>> {
        self.setAsUnfreshInner()
    }
}

struct Client {
    refresh: Box<dyn Fn()>,
    _unsubscribe: RefCell<Option<Unsubscribe>>,
}

impl Client {
    fn new<T: 'static>(computed: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Rc<Client> {
        let refresh = Box::new(move || {
            let value = computed.getValue();
            call(value);
        });
        
        refresh();

        Rc::new(
            Client {
                refresh,
                _unsubscribe: RefCell::new(None),
            }
        )
    }

    fn setUnsubscribe(&self, unsubscribe: Unsubscribe) {
        let mut inner = self._unsubscribe.borrow_mut();
        if inner.is_some() {
            panic!("Nic tu nie powinno być");
        }

        *inner = Some(unsubscribe);
    }

    fn off(self: Rc<Client>) {}

    fn recalculate(&self) {
        let Client { refresh, .. } = self;
        refresh();
    }
}

fn main() {
    println!("Hello, world!");

    let val1 = Value::new(4);
    let val2 = Value::new(5);

    // let com1: Rc<Computed<i32>> = val1.toComputed();
    // let com2: Rc<Computed<i32>> = val2.toComputed();

    let sum = Computed::from2(val1.toComputed(), val2.toComputed(), |a: Rc<i32>, b: Rc<i32>| -> i32 {
        //let aa = a.as_ref();
        a.as_ref() + b.as_ref()
    });


    let subscription = sum.subscribe(Box::new(|sum: Rc<i32>| {
        println!("Suma: {}", sum);
    }));

    println!("aaa");
    val1.setValue(333);

    println!("bbb");
    val1.setValue(888);

    println!("ccc");


    subscription.off();
}
