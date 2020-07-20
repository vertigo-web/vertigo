

struct Value<T> {
    value: T,   
}

impl<T> Value<T> {
    pub fn new() -> (Value<T>, Computed<T>) {
        todo!();
    }
}

struct Computed<T> {
    get: fn() -> T,
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




let a: Value<u64> = Value::new(3);
let b: Value<u32> = Value::new(5);

let c: Value<u64> = Value.combine2(a, b, fn(u64, u32) -> u64 {
    todo!();
});


lub bardziej mobxowo
Przewaga nad jsem nawet w tej wersji jest taka, że nie da się zrobić cykliczności pomiędzy zmiennymi

let c: Value<u64> = Value.combine2(Box::new(move fn() -> u64 {

    let aValue = a.get();
    let bValue = b.get();

    todo!();
}));


c.subscribe(fn(value: u64) -> {
    println!("wynik: {}", value);
});

a.set(4);
b.set(55);

*/



//fn 

fn main() {
    println!("Hello, world!");
}
