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
mod runtime;

use codegen::generate_ohua_runtime;
use codegen::typedgen::*;
use errors::*;
use ohua_types::OhuaData;
use ohuac::OhuaProduction;
use std::env::current_dir;
use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use tempdir::TempDir;
use type_extract::TypeKnowledgeBase;

use self::proc_macro::TokenStream;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprCall, ExprPath, Stmt};

/*
 * #[ohua] name::space::algo(arg1, arg2);
 */
#[proc_macro_attribute]
pub fn ohua(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree, extract necessary information
    let ast: Stmt = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e),
    };

    let expression: Expr = match ast {
        Stmt::Expr(e) => e,
        Stmt::Semi(e, _) => e,
        _ => panic!("Encountered unsupported statement after #[ohua] macro"),
    };

    let algo_call: ExprCall = if let Expr::Call(fn_call) = expression {
        fn_call
    } else {
        panic!("The #[ohua] macro may only be applied to a function call.");
    };

    if !args.is_empty() {
        panic!("The #[ohua] macro does currently not support macro arguments.");
    }

    let algo_name: ExprPath = match *algo_call.func {
        Expr::Path(path) => path,
        _ => panic!("Malformed algorithm invocation. Expected a qualified path."),
    };
    let algo_args: Punctuated<Expr, Token![,]> = algo_call.args; // https://docs.serde.rs/syn/punctuated/index.html

    // after the initial parsing/verification, the compilation can begin
    // create a temporary directory
    // TODO: Add cfg flag to retain build artifacts from this step
    let tmp_dir = match TempDir::new("ohuac-rs") {
        Ok(dir) => dir.into_path(),
        Err(io_err) => panic!("Unable to create a temp directory. {}", io_err),
    };

    // The compilation itself is a 4-Step Pipeline:
    /* 1. Run `ohuac` w/o optimizations
     * 2. Run Type extraction
     * 3. Run `ohuac` w/ optimizations (not yet implemented)
     * 4. Run the code generation
     */

    // Phase 1: Run `ohuac` (there are no optimizations for the moment)
    println!("[Phase 1] Starting `ohuac`");
    let ohuac_file = locate_ohuac_file(algo_name)
        .expect("The ohuac file could not be found at the requested place.");
    let processed_algo = ohuac::generate_dfg(ohuac_file, tmp_dir.clone());

    // Phase 2: Run the type extraction
    // println!("[Phase 2] Running type extraction");
    // let type_infos = match TypeKnowledgeBase::generate_from(&processed_algo) {
    //     Ok(info) => info,
    //     Err(e) => panic!("{}", e),
    // };

    // Phase 3: Run `ohuac` w/ optimizations (unimplemented as of now)
    // TODO

    // Phase 4: Run the codegen
    print!("[Phase 4] Generating Code...");
    let dfg_file = File::open(&processed_algo.ohuao).unwrap();
    let ohua_data: OhuaData = serde_json::from_reader(dfg_file).unwrap();
    println!("{}", &ohua_data);

    // all parsed code parts are unwrapped here, errors should not occur, as we've generated this
    let final_code = generate_code(&ohua_data);
    println!(" Done!");

    println!("{}", final_code.clone().to_string());
    // Hand the output tokens back to the compil)er
    final_code.into() // this converts from proc_macro2::TokenStream to proc_macro::TokenStream
}

fn locate_ohuac_file(path: syn::ExprPath) -> Option<PathBuf> {
    let mut lookup_path: Vec<String> = path
        .path
        .segments
        .iter()
        .map(|ref x| x.ident.to_string())
        .collect();

    let mut ohuac_path = current_dir().unwrap();

    // We always look *inside* the src dir
    ohuac_path.push("src");

    println!("Current dir: {:?}", ohuac_path);

    for p in lookup_path.drain(..) {
        ohuac_path.push(format!("{}", p));
    }

    ohuac_path.set_extension("ohuac");

    print!("Inspecting path: {:?} ", ohuac_path);

    if ohuac_path.exists() {
        println!("found!");
        Some(ohuac_path)
    } else {
        println!("not found.");
        None
    }
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
