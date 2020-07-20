

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



//fn 

fn main() {
    println!("Hello, world!");
}
