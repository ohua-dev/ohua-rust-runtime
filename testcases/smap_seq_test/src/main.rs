#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod smapping;

use ohua_codegen::ohua;

fn main() {
    #[ohua]
    smap_test();
}
