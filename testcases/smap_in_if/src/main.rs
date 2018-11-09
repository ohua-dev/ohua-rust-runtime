#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod smap_supplement;

use ohua_codegen::ohua;

fn main() {
    #[ohua]
    let x = smap_in_if();

    assert!(x == vec!["I hate giant spiders", "Why are there everywhere giant spiders", "there is a huge pile of giant spiders"]);
}
