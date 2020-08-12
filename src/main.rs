#![allow(non_snake_case)]

use std::collections::HashMap;
use std::collections::HashSet;
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

    fn getWithContext<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let value = self.value.borrow();
        let state = &*value;
        getter(&state, data)
    }

    fn change<D, R>(&self, data: D, changeFn: fn(&mut T, D) -> R) -> R {
        let value = self.value.borrow_mut();
        let mut state = value;
        changeFn(&mut state, data)
    }
}
struct Graph {
    relations: HashMap<u64, u64>,                   //relacje zaleności, target -> parent
    revertRelations: HashMap<u64, Vec<u64>>,        //wykorzystywane do powiadamiania o konieczności przeliczenia
                                                    //parent -> Vec<target>
}

impl Graph {
    fn new() -> Graph {
        Graph {
            relations: HashMap::new(),
            revertRelations: HashMap::new(),
        }
    }
    fn addRelation(&mut self, parentId: u64, clientId: u64) {
        self.relations.insert(clientId, parentId);

        let list = self.revertRelations.entry(parentId).or_insert_with(Vec::new);
        list.push(clientId);
    }

    fn removeRelation(&mut self, clientId: u64) {
        self.relations.remove(&clientId);
        self.revertRelations.retain(|_k, listIds| {
            listIds.retain(|item| {
                let matchId = clientId == *item;
                let shouldStay = !matchId;
                shouldStay
            });

            listIds.len() > 0
        });
    }

    fn getAllDeps(&self, parentId: u64) -> HashSet<u64> {
        let mut result = HashSet::new();
        let mut toTraverse: Vec<u64> = vec!(parentId);

        loop {
            let nextToTraverse = toTraverse.pop();

            match nextToTraverse {
                Some(next) => {
                    result.insert(next);

                    let list = self.revertRelations.get(&next);

                    if let Some(list) = list {

                        for item in list {
                            let isContain = result.contains(item);
                            if isContain {
                                //ignore
                            } else {

                                toTraverse.push(*item);
                            }
                        }
                    }
                },
                None => {
                    return result;
                }
            }
        }
    }
}

struct DependenciesInner {
    computed: HashMap<u64, ComputedRefresh>,        //To wykorzystujemy do wytrigerowania odpowiednich akcji
    client: HashMap<u64, ClientRefresh>,            //To wykorzystujemy do wytrigerowania odpowiedniej reakcji
    graph: Graph,
}

impl DependenciesInner {
    fn new() -> DependenciesInner {
        DependenciesInner {
            computed: HashMap::new(),
            client: HashMap::new(),
            graph: Graph::new(),
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

    fn triggerChange(self: &Rc<Dependencies>, parentId: u64) {

        self.inner.getWithContext(parentId, |state, parentId| {
            let allDeps = state.graph.getAllDeps(parentId);

            for itemId in allDeps.iter() {
                let item = state.computed.get(itemId);

                if let Some(item) = item {
                    item.setAsUnfreshInner();
                }
            }

            for itemId in allDeps.iter() {
                let item = state.client.get(itemId);

                if let Some(item) = item {
                    item.recalculate();
                }
            }
        });
    }

    fn addRelation(self: &Rc<Dependencies>, parentId: u64, client: ComputedRefresh) {
        self.inner.change((parentId, client), |state, (parentId, client)| {
            let clientId = client.getId();
            state.computed.insert(clientId, client);

            state.graph.addRelation(parentId, clientId);
        });
    }

    fn addRelationToClient(self: &Rc<Dependencies>, parentId: u64, client: ClientRefresh) {
        self.inner.change((parentId, client), |state, (parentId, client)| {
            let clientId = client.getId();
            state.client.insert(clientId, client);

            state.graph.addRelation(parentId, clientId);
        });
    }

    fn removeRelation(self: &Rc<Dependencies>, clientId: u64) {
        self.inner.change(clientId, |state, clientId| {
            state.computed.remove(&clientId);
            state.client.remove(&clientId);

            state.graph.removeRelation(clientId);
        });
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

        let computed = Computed::new(self.deps.clone(), getValue);

        self.deps.addRelation(self.id, computed.getComputedRefresh());

        computed
    }
}


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
    deps: Rc<Dependencies>,
    id: u64,
    refresh: Rc<BoxRefCell<Box<dyn Fn()>>>,
}

impl Client {
    fn new<T: Debug + 'static>(deps: Rc<Dependencies>, computed: Rc<Computed<T>>, call: Box<dyn Fn(Rc<T>) + 'static>) -> Client {
        let refresh = Box::new(move || {
            let value = computed.getValue();
            call(value);
        });
        
        refresh();

        Client {
            deps,
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
        self.deps.removeRelation(self.id);
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