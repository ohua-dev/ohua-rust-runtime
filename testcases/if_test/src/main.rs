#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod iftest;

use ohua_codegen::ohua;

fn main() {
    let ctrl_input = true;
    let splicable_input = String::from("executed: ");

    #[ohua]
    let result = if_test(ctrl_input, splicable_input);

    assert!(result == "executed: yes");
}
