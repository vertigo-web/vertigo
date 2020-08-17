#![allow(non_snake_case)]

mod lib;
mod vdom;
#[cfg(test)]
mod tests;

/*
TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)
*/

fn main() {
    println!("test");
}
