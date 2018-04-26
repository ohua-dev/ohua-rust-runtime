//! The Rust Ohua Runtime Generator.
//!
//! This program generates a rust runtime from an [Ohua](https://github.com/ohua-dev) algorithm, which can be defined in an `ohuac` file.
//!
//! TODO: Expand me! (Issue: [#15](https://github.com/ohua-dev/ohua-rust-runtime/issues/15))

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate syn;
extern crate quote;
extern crate tempdir;

pub mod errors;
pub mod ohua_types;
pub mod type_extract;
pub mod ohuac;
pub mod codegen;

use codegen::generate_ohua_runtime;
use errors::*;
use type_extract::TypeKnowledgeBase;
use tempdir::TempDir;
use std::io;
use std::path::PathBuf;
use std::fs;
use std::env::current_dir;
use std::error::Error;

/// Recursively searches all subdirectories for `.ohuac` files
pub fn find_ohuac_files(
    current_path: PathBuf,
    mut found: Vec<PathBuf>,
) -> io::Result<Vec<PathBuf>> {
    // iterate over all files in a folder, recurse deeper if it's a folder
    for entry in fs::read_dir(current_path)? {
        let cur_path = entry?.path();
        if cur_path.is_dir() {
            found = find_ohuac_files(cur_path, found)?;
        } else if Some("ohuac".as_ref()) == cur_path.extension() {
            found.push(cur_path);
        }
    }

    Ok(found)
}

/// Convenience wrapper to run the build process by calling a single function. For easy use from within a `build.rs` file.
pub fn run_ohua_build() {
    let tmp_dir = match TempDir::new("ohuac-rs") {
        Ok(dir) => dir.into_path(),
        Err(io_err) => panic!("Unable to create a temp directory. {}", io_err),
    };

    // search for all ohuac files in the project folder
    // NOTE: `current_dir()` returns the project dir, from where cargo operates!
    let sources =
        find_ohuac_files(current_dir().unwrap(), vec![]).expect("Failed to locate `.ohuac` files.");
    if sources.is_empty() {
        return;
    }

    // TODO: 4-Step Pipeline
    /* 1. Run `ohuac` w/o optimizations
     * 2. Run Type extraction
     * 3. Run `ohuac` w/ optimizations (not yet implemented)
     * 4. Run the code generation
     */

    // Phase 1: Run `ohuac` (there are no optimizations for the moment)
    let processed_algos = ohuac::generate_dfgs(sources, tmp_dir.clone());

    // Phase 2: Run the type extraction
    for algo in &processed_algos {
        let type_infos = match TypeKnowledgeBase::generate_from(algo) {
            Ok(info) => info,
            Err(e) => panic!("{}", e),
        };

        println!("Knowledge Base: {:#?}", type_infos);
    }

    // Phase 3: Run `ohuac` w/ optimizations (unimplemented)
    // TODO

    // Phase 4: Run the codegen
    let mut target_dir = current_dir().unwrap();
    target_dir.push("src/");

    for algo in processed_algos {
        // TODO: Check algos don't occur twice
        // TODO: name algo-folders
        // TODO: Include `ohuac` into CI
        let algo_target = String::from(target_dir.to_str().unwrap());
        if let Err(e) = generate_ohua_runtime(algo.ohuao, algo.typedump, algo_target) {
            panic!("Code generation failed! {}", e.description());
        }
    }
}
