#![feature(proc_macro_expr)]

extern crate ohua_rust_runtime;

mod addition;

use ohua_rust_runtime::ohua;

pub fn main() {
    #[ohua] foobar();
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
