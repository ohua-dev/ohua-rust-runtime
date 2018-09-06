#![feature (proc_macro_expr)]

extern crate ohua_rust_runtime;


pub fn main() {
    #[ohua] some_algo(param1);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn ohua_macro_test() {
//         #[ohua] some_algo(param1);
//     }
// }
