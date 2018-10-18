#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod addition;

use ohua_codegen::ohua;

pub fn main() {
    #[ohua]
    foobar();
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
