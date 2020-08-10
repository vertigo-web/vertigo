#![allow(non_snake_case)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

trait Subscriber {
    fn recalculate(&self);
}

trait Observer {
    fn call(&self) -> Vec<Box<dyn Subscriber>>;
    fn getId(&self) -> u64;
}

struct Unsubscribe {
    parent: Rc<Subscription>,
    client: Rc<dyn Observer>,
}

impl Unsubscribe {
    fn new(parent: Rc<Subscription>, client: Rc<dyn Observer>) -> Unsubscribe {
        Unsubscribe {
            parent,
            client
        }
    }
}

impl Drop for Unsubscribe {
    fn drop(&mut self) {
        let Unsubscribe { parent, client } = self;
        parent.remove(client);
    }
}

struct Subscription {
    list: RefCell<HashMap<u64, Rc<dyn Observer>>>,
}

impl Subscription {
    pub fn new() -> Rc<Subscription> {
        Rc::new(Subscription {
            list: RefCell::new(HashMap::new())
        })
    }

    pub fn add(self: &Rc<Subscription>, observer: Rc<dyn Observer>) -> Unsubscribe {
        let id = observer.getId();
        let mut list = self.list.borrow_mut();
        let result = list.insert(id, observer.clone());

        if result.is_some() {
            panic!("Coś poszło nie tak");
        }

        Unsubscribe::new(self.clone(), observer.clone())
    }

    pub fn trigger(&self) -> Vec<Box<dyn Subscriber>> {
        let mut out: Vec<Box<dyn Subscriber>> = Vec::new();
        let mut list = self.list.borrow();
        for (_, item) in list.iter() {
            let mut subList = item.call();
            out.append(&mut subList);
        }

        out
    }

    pub fn remove(self: &Rc<Subscription>, observer: &Rc<dyn Observer>) {
        let id = observer.getId();
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

    pub fn setValue(self: &Rc<Value<T>>, value: T) -> Vec<Box<dyn Subscriber>> {                          //TODO - trzeba odebrać i wywołać
        let mut inner = self.refCell.borrow_mut();
        inner.value = Rc::new(value);
        self.subscription.trigger()
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

        let unsubscribe = self.subscription.add(computed.clone());
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
    getValue: Box<dyn Fn() -> Rc<T> + 'static>,
    refCell: RefCell<ComputedValue<T>>,
}

impl<T: 'static> Computed<T> {
    pub fn new<F: Fn() -> T + 'static>(getValue: Box<F>) -> Rc<Computed<T>> {
        let newGetValue = Box::new(move || {
            Rc::new(getValue())
        });

        let value = newGetValue();

        Rc::new(
            Computed {
                getValue: newGetValue,
                refCell: ComputedValue::new(value),
            }
        )
    }

    pub fn newRc<F: Fn() -> Rc<T> + 'static>(getValue: Box<F>) -> Rc<Computed<T>> {
        let value = getValue();
        Rc::new(
            Computed {
                getValue: getValue,
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

        let aUnsubscribe = a.addSubscription(result.clone());
        let bUnsubscribe = b.addSubscription(result.clone());

        result.addToUnsubscribeList(aUnsubscribe);
        result.addToUnsubscribeList(bUnsubscribe);

        result
    }

    pub fn getValue(&self) -> Rc<T> {
        let mut inner = self.refCell.borrow_mut();

        if inner.isFresh == false {
            inner.value = self.getValue();
            inner.isFresh = true;
        }

        inner.value.clone()
    }

    pub fn setAsUnfresh(&self) -> Vec<Box<dyn Subscriber>> {
        let mut inner = self.refCell.borrow_mut();
        inner.isFresh = false;
        inner.subscription.trigger()
    }

    pub fn addSubscription(&self, observer: Rc<dyn Observer>) -> Unsubscribe {
        let inner = self.refCell.borrow_mut();
        let unsubscribe = inner.subscription.add(observer);
        unsubscribe
    }

    pub fn subscribe(self: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Rc<Client> {
        let client = Client::new(self.clone(), call);

        let unsubscribe = self.addSubscription(client.clone());

        client.setUnsubscribe(unsubscribe);

        client
    }
}

impl<T> Observer for Computed<T> {
    fn call(&self) -> Vec<Box<dyn Subscriber>> {
        self.setAsUnfresh()
    }

    fn getId(&self) -> u64 {
        todo!();
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
}

impl Observer for Client {
    fn call(&self) -> Vec<Box<dyn Subscriber>> {
        todo!();
    }

    fn getId(&self) -> u64 {
        todo!();
    }
}

impl Subscriber for Client {
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
