#![feature(proc_macro_expr)]

extern crate ohua_rust_runtime;

use ohua_rust_runtime::ohua;

pub fn main() {
    let param = 12;

    #[ohua]
    foobar(param);
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
