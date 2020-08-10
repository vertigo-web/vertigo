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
    parent: Subscription,
    client: Rc<dyn Observer>,
}

impl Drop for Unsubscribe {
    fn drop(&mut self) {
        let Unsubscribe { parent, client } = self;
        parent.remove(client);
    }
}

struct Subscription {
    list: HashMap<u64, Rc<dyn Observer>>,
}

impl Subscription {
    pub fn new() -> Subscription {
        Subscription {
            list: HashMap::new()
        }
    }

    pub fn add(&mut self, observer: Rc<dyn Observer>) -> Unsubscribe {
        let id = observer.getId();
        let result = self.list.insert(id, observer);

        if result.is_some() {
            panic!("Coś poszło nie tak");
        }
    }

    pub fn trigger(&self) -> Vec<Box<dyn Subscriber>> {
        let mut out: Vec<Box<dyn Subscriber>> = Vec::new();

        for (_, item) in self.list.iter() {
            let mut subList = item.call();
            out.append(&mut subList);
        }

        out
    }

    pub fn remove(&mut self, observer: &Rc<dyn Observer>) {
        let id = observer.getId();
        let result = self.list.remove(&id);

        if result.is_none() {
            panic!("Błąd usuwania");
        }
    }
}

struct Value<T: 'static> {
    value: Rc<T>,
    subscription: Subscription,
}

impl<T: 'static> Value<T> {
    pub fn new(value: T) -> Rc<Value<T>> {
        Rc::new(Value {
            value: Rc::new(value),
            subscription: Subscription::new(),
        })
    }

    pub fn setValue(&mut self, value: T) -> Vec<Box<dyn Subscriber>> {                          //TODO - trzeba odebrać i wywołać
        self.value = Rc::new(value);
        self.subscription.trigger()
    }

    pub fn getValue(&self) -> Rc<T> {
        self.value.clone()
    }

    pub fn toComputed(self: &Rc<Value<T>>) -> Rc<Computed<T>> {
        let selfClone = self.clone();

        let getValue = Box::new(move || {
            selfClone.getValue()
        });

        Computed::newRc(getValue)
    }
}

struct ComputedValue<T: 'static> {
    isFresh: bool,
    value: Rc<T>,
    subscription: Subscription
}

impl<T: 'static> ComputedValue<T> {
    pub fn new(value: Rc<T>) -> RefCell<ComputedValue<T>> {
        RefCell::new(ComputedValue {
            isFresh: true,
            value,
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

    pub fn from2<A, B, R>(
        a: Rc<Computed<A>>,
        b: Rc<Computed<B>>,
        calculate: fn(Rc<A>, Rc<B>) -> R
    ) -> Rc<Computed<R>> {

        //TODO - dodać subskrybcje ...

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

        a.addSubscription(result.clone());
        b.addSubscription(result.clone());

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

    pub fn addSubscription(&self, observer: Rc<dyn Observer>) {
        let mut inner = self.refCell.borrow_mut();
        inner.subscription.add(observer);
    }

    pub fn subscribe(self) -> Client {
        todo!();
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

impl<T> Drop for Computed<T> {
    fn drop(&mut self) {

        //TODO - odsybskrybować
        todo!();
    }
}

struct Client {
    refresh: Box<dyn Fn()>,
}

impl Client {
    fn new<T: 'static>(getValue: Box<dyn Fn() -> Rc<T> + 'static>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Client {
        let refresh = Box::new(move || {
            let value = getValue();
            call(value);
        });
        
        Client {
            refresh,
        }
    }
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
        let Client { refresh } = self;
        refresh();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        //TODO - odsybskrybować
        todo!();
    }
}

fn main() {
    println!("Hello, world!");

    let a = 3;
}
