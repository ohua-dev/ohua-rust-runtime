#![feature(proc_macro_hygiene, fnbox)]
extern crate ohua_codegen;
extern crate ohua_runtime;

mod house;

use house::{EmotionalState, House, Human};
use ohua_codegen::ohua;

fn main() {
    #[ohua]
    custom_types();
}
