

//https://crates.io/crates/simple-mutex


struct Context {
}

impl Context {
    pub fn new<T>() -> Value<T> {
        todo!();
    }

    pub fn calculate<K>(fun: Box<dyn Fn() -> K>) -> Computed<K> {
        todo!();
    }
}

struct Value<T> {
    value: T,   
}

impl<T> Value<T> {
    pub fn toComputed(&self) -> Computed<T> {
        todo!();
    }
}

struct Computed<T> {
    get: Box<dyn Fn() -> T>,
}

impl<T> Computed<T> {
}


/*
                                fn() ->             tylko czyste funkcje

Value --> T
Value --> K

fn(
    fn() -> T,          //subskrybcja podczas obliczania
    fn() -> K
) -> R



Value =====>    impl ValueTrait

    get() -> T                                      === pobiera oraz subskrybuje



let context: Context = Context::new();

let a: Value<u64> = context.value(3);
let b: Value<u32> = context.value(5);


lub bardziej mobxowo
Przewaga nad jsem nawet w tej wersji jest taka, że nie da się zrobić cykliczności pomiędzy zmiennymi

let c: Computed<u64> = context.combine2(Box::new(move fn(&mut context) -> u64 {

    let aValue = context.get(a);    //,.get();
    let bValue = context.get(b);    //.get();

    todo!();
}));


context.subscribe(c, fn(value: u64) -> {
    println!("wynik: {}", value);
});

context.set(a, 4);
context.set(b, 55);


Component
    render() {

    }

    Component
        render() {

        }

*/



//fn 

fn main() {
    println!("Hello, world!");
}
