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

use std::fs::{File, DirBuilder};
use std::io::{self, Write};
use std::process;
use clap::{App, Arg};
use types::OhuaData;
use runtime_data::generate_runtime_data;


fn populate_static_files(path: String) -> io::Result<()> {
    let mod_file = include_bytes!("templates/mod.rs");
    File::create(path.clone() + "/mod.rs")?.write_all(mod_file)?;

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
        .arg(Arg::with_name("target")
                 .help("Target path for the Ohua runtime.")
                 .required(true)
                 .takes_value(true))
        .get_matches();

    let path = matches.value_of("input").unwrap();
    let mut output = String::from(matches.value_of("target").unwrap());

    // TODO: Remove recursive directory creation
    if let Err(err) = DirBuilder::new().recursive(true).create(output.as_str()) {
        eprintln!("[Error] Unable to create the target directory. {}", err);
        process::exit(1);
    }

    // ====== start the real work ======

    // read the data structure
    let file = File::open(path).unwrap();
    let ohua_data: OhuaData = serde_json::from_reader(file).unwrap();

    // generate the module of the ohua runtime and populate it with the static files
    output += "/ohua_runtime";
    if let Err(err) = DirBuilder::new().create(output.as_str()) {
        eprintln!("[Error] Unable to create the module directory for the ohua runtime. {}", err);
        process::exit(1);
    }

    if let Err(err) = populate_static_files(output.clone()) {
        eprintln!("[Error] The static `ohua_runtime` module folder population failed unexpectedly. {}", err);
        process::exit(1);
    }

    // generate all necessary type casts for the Arcs
    if let Err(err) = typecasts::generate_casts(&ohua_data.graph.operators, (output.clone() + "/generictype.rs").as_str()) {
        eprintln!("[Error] Unable to create the generic type file. {}", err);
        process::exit(1);
    }

    // generate wrapper functions for all operators
    let altered_ohuadata = match wrappers::generate_wrappers(ohua_data, (output.clone() + "/wrappers.rs").as_str()) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("[Error] Unable to create the function wrappers. {}", err);
            process::exit(1);
        }
    };

    // write the runtime OhuaData structure
    if let Err(err) = generate_runtime_data(altered_ohuadata, (output + "/runtime.rs").as_str()) {
        eprintln!("[Error] Unable to create the runtime data structure file. {}", err);
        process::exit(1);
    }
}
