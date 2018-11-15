#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod iftest;

use ohua_codegen::ohua;

fn main() {
    let inp = String::from("executed: ");

    #[ohua]
    let result = if_test(true, inp);

    assert!(result == "executed: yes");
}
