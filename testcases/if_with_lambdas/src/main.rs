#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod calculations;

use ohua_codegen::ohua;

fn main() {
    #[ohua]
    let x = lambda_test();

    assert!(x == 8);
}
