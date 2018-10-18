#![feature(proc_macro_hygiene, fnbox)]
extern crate ohua_codegen;
extern crate ohua_runtime;

use ohua_codegen::ohua;

mod mainclone;

fn main() {
    #[ohua]
    mainargs(15);
}
