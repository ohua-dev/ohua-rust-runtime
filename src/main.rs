#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod types;
mod typecasts;
mod wrappers;

use std::fs::File;
use clap::{App, Arg};
use types::OhuaData;

fn main() {
    let matches = App::new("ohua-rust-runtime")
        .version(crate_version!())
        .author("Felix Wittwer <dev@felixwittwer.de>")
        .about("Rust runtime generation from Ohua code.")
        .arg(Arg::with_name("input")
                 .help("Ohua object file in JSON format.")
                 .required(true)
                 .takes_value(true))
        .get_matches();

    let path = matches.value_of("input").unwrap();
    let file = File::open(path).unwrap();

    let ohua_data: OhuaData = serde_json::from_reader(file).unwrap();

    // typecasts::generate_casts(vec!["&str", "u16", "bool"], "");

    // wrappers::wrap_function("hello::world", 4, 3);

    let altered_ohuadata = wrappers::generate_wrappers(ohua_data, "wrappers.rs");

    println!("{}", altered_ohuadata);

    // TODO:
    // - alter the ohua_data structure: analyze the structure and output an altered
    //      version that handles arguments etc. [runtime.rs]
    // - generate the function wrapper code (requires information about the number
    //      of function arguments) [wrappers.rs]
    // - generate typecasts (requires information about the datatypes involved)
    //      [generictype.rs]
}
