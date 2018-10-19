#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod reuse_foo;

use ohua_codegen::ohua;

fn main() {
    let text = String::from("the quick brown fox jumped");

    #[ohua]
    let result = reuse(text);

    println!("Computation result: {}", result);
}
