#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod tuple_ret;

use ohua_codegen::ohua;

fn main() {
    let input = String::from("This is a test string.");
    #[ohua]
    let (old, spliced, new) = tuple_return(input);

    assert!(old < new);
    println!(
        "Old/new string length: {}/{} -- String: \"{}\"",
        old, new, spliced
    );
}
