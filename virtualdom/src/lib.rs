#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod computed;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


pub fn test(a: i32, b: i32) -> i32 {
    println!("Test z bibiliteki");
    a + b
}


