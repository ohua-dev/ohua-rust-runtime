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
use std::process;
use clap::{App, Arg};
use types::OhuaData;
use runtime_data::generate_runtime_data;

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
    let output = String::from(matches.value_of("target").unwrap());

    if let Err(err) = DirBuilder::new().recursive(true).create(output.as_str()) {
        println!("[Error] Unable to create the target directory. {}", err);
        process::exit(1);
    }

    // ====== start the real work ======

    // read the data structure
    let file = File::open(path).unwrap();
    let ohua_data: OhuaData = serde_json::from_reader(file).unwrap();

    // generate all necessary type casts for the Arcs
    if let Err(err) = typecasts::generate_casts(&ohua_data.graph.operators, (output.clone() + "/generictype.rs").as_str()) {
        println!("[Error] Unable to create the generic type file. {}", err);
        process::exit(1);
    }

    // generate wrapper functions for all operators
    let altered_ohuadata = match wrappers::generate_wrappers(ohua_data, (output.clone() + "/wrappers.rs").as_str()) {
        Ok(data) => data,
        Err(err) => {
            println!("[Error] Unable to create the function wrappers. {}", err);
            process::exit(1);
        }
    };

    // write the runtime OhuaData structure
    if let Err(err) = generate_runtime_data(&altered_ohuadata, (output + "/runtime.rs").as_str()) {
        println!("[Error] Unable to create the runtime data structure file. {}", err);
        process::exit(1);
    }
}
