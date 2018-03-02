//! # The Rust Ohua Runtime Generator
//!
//! This program generates a rust runtime for an [Ohua](https://github.com/ohua-dev) program, which can be defined in an `ohuac` file.

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod types;
mod typecasts;
mod wrappers;
mod runtime_data;
mod modgen;

use std::fs::{File, DirBuilder};
use std::io::{self, Write};
use std::process;
use clap::{App, Arg};
use types::{OhuaData, AlgorithmArguments};
use runtime_data::generate_runtime_data;


/// This function writes all static files to their respective locations,
/// returning an error when the write operation exitted unsuccessfully.
fn populate_static_files(path: String) -> io::Result<()> {
    let type_file = include_bytes!("templates/types.rs");
    File::create(path.clone() + "/types.rs")?.write_all(type_file)?;

    Ok(())
}

fn main() {
    let matches = App::new("ohua-rust-runtime")
        .version(crate_version!())
        .author("Felix Wittwer <dev@felixwittwer.de>")
        .about("Rust runtime generation from Ohua code.")
        .arg(Arg::with_name("input")
                 .help("Ohua object file in JSON format.")
                 .required(true)
                 .takes_value(true))
        .arg(Arg::with_name("typeinfo")
                 .help("Ohua `type_dump` file in JSON format.")
                 .required(true)
                 .takes_value(true))
        .arg(Arg::with_name("target")
                 .help("Target path for the Ohua runtime.")
                 .required(true)
                 .takes_value(true))
        .get_matches();

    let dfg_path = matches.value_of("input").unwrap();
    let typeinfo_path = matches.value_of("typeinfo").unwrap();
    let mut output = String::from(matches.value_of("target").unwrap());

    // TODO: Remove recursive directory creation
    if let Err(err) = DirBuilder::new().recursive(true).create(output.as_str()) {
        eprintln!("[Error] Unable to create the target directory. {}", err);
        process::exit(1);
    }

    // ====== start the real work ======

    // read the data structures
    let dfg_file = File::open(dfg_path).unwrap();
    let ohua_data: OhuaData = serde_json::from_reader(dfg_file).unwrap();
    let typeinfo_file = File::open(typeinfo_path).unwrap();
    let type_data: AlgorithmArguments = serde_json::from_reader(typeinfo_file).unwrap();

    // check whether both mainArity and number of argument types match
    if ohua_data.mainArity as usize != type_data.argument_types.len() {
        eprintln!("[Error] The number of arguments specified in the OhuaData structure and the `type_dump` file don't match.");
        process::exit(1);
    }

    // generate the module of the ohua runtime
    output += "/ohua_runtime";
    if let Err(err) = DirBuilder::new().create(output.as_str()) {
        eprintln!("[Error] Unable to create the module directory for the ohua runtime. {}", err);
        process::exit(1);
    }

    // populate the module with the static files
    if let Err(err) = populate_static_files(output.clone()) {
        eprintln!("[Error] The static `ohua_runtime` module folder population failed unexpectedly. {}", err);
        process::exit(1);
    }

    // generate all necessary type casts for the Arcs
    if let Err(err) = typecasts::generate_casts(&ohua_data.graph.operators, &type_data, (output.clone() + "/generictype.rs").as_str()) {
        eprintln!("[Error] Unable to create the generic type file. {}", err);
        process::exit(1);
    }

    // generate wrapper functions for all operators
    let altered_ohuadata = match wrappers::generate_wrappers(ohua_data, &type_data.argument_types, (output.clone() + "/wrappers.rs").as_str()) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[Error] Unable to create the function wrappers. {}", err);
            process::exit(1);
        }
    };

    if let Err(err) = modgen::generate_modfile(&type_data, (output.clone() + "/mod.rs").as_str()) {
        eprintln!("[Error] Unable to create the module file. {}", err);
        process::exit(1);
    }

    // write the runtime OhuaData structure
    if let Err(err) = generate_runtime_data(altered_ohuadata, (output + "/runtime.rs").as_str()) {
        eprintln!("[Error] Unable to create the runtime data structure file. {}", err);
        process::exit(1);
    }
}
