//! The Rust Ohua Runtime Generator.
//!
//! This program generates a rust runtime from an [Ohua](https://github.com/ohua-dev) algorithm, which can be defined in an `ohuac` file.
//!
//! TODO: Expand me!

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod codegen;
pub mod comp_errors;

use codegen::generate_ohua_runtime;
use comp_errors::*;

pub fn run_build() {
    // TODO: 4-Step Pipeline
    /* 1. Run `ohuac` w/o optimizations
     * 2. Run Type extraction
     * 3. Run `ohuac` w/ optimizations (not yet implemented)
     * 4. Run the code generation
    */
    println!("Boom.");
}
