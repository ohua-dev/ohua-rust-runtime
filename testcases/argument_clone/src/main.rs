#![feature(proc_macro_hygiene)]
extern crate ohua_codegen;
extern crate ohua_runtime;

use ohua_codegen::ohua;

mod strings;

fn main() {
    #[ohua] argument_clone();
}
