//! The Rust Ohua Runtime Generator.
//!
//! This program generates a rust runtime from an [Ohua](https://github.com/ohua-dev) algorithm, which can be defined in an `ohuac` file.
//!
//! TODO: Expand me! (Issue: [#15](https://github.com/ohua-dev/ohua-rust-runtime/issues/15))
#![allow(dead_code, unused_imports, unused_variables)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate tempdir;

extern crate proc_macro;
extern crate proc_macro2;

mod codegen;
mod errors;
mod ohua_types;
mod ohuac;
mod type_extract;

use codegen::generate_ohua_runtime;
use codegen::typedgen::*;
use errors::*;
use ohuac::OhuaProduction;
use std::env::current_dir;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use tempdir::TempDir;
use type_extract::TypeKnowledgeBase;

use self::proc_macro::TokenStream;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprCall};

/*
 * #[ohua] algo(arg1, arg2);
 */
#[proc_macro_attribute]
pub fn ohua(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast: Expr = syn::parse(input).unwrap();

    let algo_call: ExprCall = if let Expr::Call(fn_call) = ast {
        fn_call
    } else {
        panic!("The #[ohua] may only be applied to a function call.");
    };

    if !args.is_empty() {
        panic!("The #[ohua] macro does currently not support macro arguments.");
    }

    let algo_name: Box<Expr> = algo_call.func;
    let algo_args: Punctuated<Expr, Token![,]> = algo_call.args; // https://docs.serde.rs/syn/punctuated/index.html

    // after the initial parsing/verification, the compilation can begin
    // create a temporary directory
    // TODO: Add cfg flag to retain build artifacts from this step
    let tmp_dir = match TempDir::new("ohuac-rs") {
        Ok(dir) => dir.into_path(),
        Err(io_err) => panic!("Unable to create a temp directory. {}", io_err),
    };

    // search for all ohuac files in the project folder
    // NOTE: `current_dir()` returns the project dir, from where cargo operates!
    let sources =
        find_ohuac_files(current_dir().unwrap(), vec![]).expect("Failed to locate `.ohuac` files.");
    if sources.is_empty() {
        panic!("No ohua algorithm files were found in the crate.");
    }

    // The compilation itself is a 4-Step Pipeline:
    /* 1. Run `ohuac` w/o optimizations
     * 2. Run Type extraction
     * 3. Run `ohuac` w/ optimizations (not yet implemented)
     * 4. Run the code generation
     */

    // Phase 1: Run `ohuac` (there are no optimizations for the moment)
    // TODO Filter out the actual algo we are interested in!
    let ohuac_file = PathBuf::default();
    let processed_algo = ohuac::generate_dfg(ohuac_file, tmp_dir.clone());

    // Phase 2: Run the type extraction
    let type_infos = match TypeKnowledgeBase::generate_from(&processed_algo) {
        Ok(info) => info,
        Err(e) => panic!("{}", e),
    };

    println!("Knowledge Base: {:#?}", type_infos);

    // Phase 3: Run `ohuac` w/ optimizations (unimplemented as of now)
    // TODO

    // Phase 4: Run the codegen

    // TODO Felix: - locate (meh) and load (x) the algo file
    //             - run the ohua-core compiler to generate the output (catch it as a string) (ok)
    //             - create the OhuaData structure from the compiler output
    let compiled_ohua = unimplemented!();
    // let stream = TokenStream::new();
    // let tokens: Result<TokenStream, LexError> = stream.from_str(&code);

    // TODO relocated the overall structure into a quote
    // let header_code = unimplemented!();
    let arc_code = generate_arcs(compiled_ohua);
    let op_code = generate_sfns(compiled_ohua); // Vec<String>
    let final_code = quote!{
        // FIXME we can not have header code. all functions/identifiers need to be fully qualified.
        // #header_code
        {
            #arc_code

            #op_code

            run_ohua(tasks)
        }
    };

    // Hand the output tokens back to the compiler
    final_code.into()
}

/// Recursively searches all subdirectories for `.ohuac` files
fn find_ohuac_files(current_path: PathBuf, mut found: Vec<PathBuf>) -> io::Result<Vec<PathBuf>> {
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

// TODO: To be retired
// /// Convenience wrapper to run the build process by calling a single function. For easy use from within a `build.rs` file.
// fn run_ohua_build() {
//     let tmp_dir = match TempDir::new("ohuac-rs") {
//         Ok(dir) => dir.into_path(),
//         Err(io_err) => panic!("Unable to create a temp directory. {}", io_err),
//     };

//     // search for all ohuac files in the project folder
//     // NOTE: `current_dir()` returns the project dir, from where cargo operates!
//     let sources =
//         find_ohuac_files(current_dir().unwrap(), vec![]).expect("Failed to locate `.ohuac` files.");
//     if sources.is_empty() {
//         return;
//     }

//     // TODO: 4-Step Pipeline
//     /* 1. Run `ohuac` w/o optimizations
//      * 2. Run Type extraction
//      * 3. Run `ohuac` w/ optimizations (not yet implemented)
//      * 4. Run the code generation
//      */

//     // Phase 1: Run `ohuac` (there are no optimizations for the moment)
//     let mut processed_algos = ohuac::generate_dfgs(sources, tmp_dir.clone());

//     // Phase 2: Run the type extraction
//     let mut algo_info: Vec<(OhuaProduction, TypeKnowledgeBase)> =
//         Vec::with_capacity(processed_algos.len());
//     for algo in processed_algos.drain(..) {
//         let type_infos = match TypeKnowledgeBase::generate_from(&algo) {
//             Ok(info) => info,
//             Err(e) => panic!("{}", e),
//         };

//         println!("Knowledge Base: {:#?}", type_infos);
//         algo_info.push((algo, type_infos));
//     }

//     // Phase 3: Run `ohuac` w/ optimizations (unimplemented)
//     // TODO

//     // Phase 4: Run the codegen
//     let mut target_dir = current_dir().unwrap();
//     target_dir.push("src/");

//     for &(ref algo, ref info) in &algo_info {
//         // TODO: Check algos don't occur twice
//         // TODO: name algo-folders
//         let algo_target = String::from(target_dir.to_str().unwrap());
//         if let Err(e) = generate_ohua_runtime(&algo, algo_target, &info) {
//             panic!("Code generation failed! {}", e.description());
//         }
//     }
// }
