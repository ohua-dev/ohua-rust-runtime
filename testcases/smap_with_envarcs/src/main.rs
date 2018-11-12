#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod smapping;

use ohua_codegen::ohua;

fn main() {
    let inputs: Vec<String> = vec![
        "I hate".into(),
        "Why are there everywhere".into(),
        "there is a huge pile of".into(),
    ];

    #[ohua]
    let x = smap_test(inputs);

    println!("Received: {:?}", x);
}
