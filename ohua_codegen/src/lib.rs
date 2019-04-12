//! The Rust Ohua Runtime Generator.
//!
//! This program generates a rust runtime from an [Ohua](https://github.com/ohua-dev) algorithm, which can be defined in an `ohuac` file.
//!
//! TODO: Expand me! (Issue: [#15](https://github.com/ohua-dev/ohua-rust-runtime/issues/15))
#![recursion_limit = "128"]

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

mod backend_optimizations;
mod errors;
mod lang;
mod ohua_types;
mod ohuac;
mod parse;
mod typedgen;

use errors::*;
use ohua_types::OhuaData;
use parse::parse_ohua_call;
use std::env::current_dir;
use std::fs::File;
use std::path::PathBuf;
use tempdir::TempDir;
use typedgen::*;

use self::proc_macro::TokenStream;
use syn::export::ToTokens;

/*
 * #[ohua] name::space::algo(arg1, arg2);
 */
#[proc_macro_attribute]
pub fn ohua(args: TokenStream, input: TokenStream) -> TokenStream {
    let (algo_info, assignment) = parse_ohua_call(args, input);
    let (algo_name, algo_args) = algo_info;

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
    #[cfg(feature = "debug")]
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
    #[cfg(feature = "debug")]
    println!("[Phase 4] Deserializing the ohuac file.");
    let dfg_file = File::open(&processed_algo.ohuao).unwrap();
    let mut ohua_data: OhuaData = match serde_json::from_reader(dfg_file) {
        Ok(data) => data,
        Err(e) => panic!("{}", e),
    };
    #[cfg(feature = "debug")]
    println!("[Phase 4] Starting code generation");
    alter_ohua_ns_imports(&mut ohua_data);

    // all parsed code parts are unwrapped here, errors should not occur, as we've generated this
    let final_code = generate_code(&mut ohua_data, &algo_args);
    #[cfg(feature = "debug")]
    println!(" Done!");

    #[cfg(feature = "debug")]
    println!("{}", final_code);
    // Hand the output tokens back to the compil)er
    if let Some(mut local) = assignment {
        local.init = Some((syn::token::Eq::default(), syn::parse2(final_code).unwrap()));
        let x = local.into_token_stream().into();
        // println!("\n\n---\n{}", x);
        x
    } else {
        let exp = syn::parse2(final_code).unwrap();
        syn::Stmt::Semi(exp, syn::token::Semi::default())
            .into_token_stream()
            .into()
    }
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

    #[cfg(feature = "debug")]
    println!("Current dir: {:?}", ohuac_path);

    for p in lookup_path.drain(..) {
        ohuac_path.push(format!("{}", p));
    }

    ohuac_path.set_extension("ohuac");

    #[cfg(feature = "debug")]
    print!("Inspecting path: {:?} ", ohuac_path);

    if ohuac_path.exists() {
        #[cfg(feature = "debug")]
        println!("found!");
        Some(ohuac_path)
    } else {
        #[cfg(feature = "debug")]
        println!("not found.");
        None
    }
}

fn alter_ohua_ns_imports(data: &mut OhuaData) {
    for op in &mut data.graph.operators {
        if op.operatorType.qbNamespace == vec!["ohua", "lang"] {
            op.operatorType.qbNamespace = vec!["ohua_runtime".into(), "lang".into()];
        }
    }
}
