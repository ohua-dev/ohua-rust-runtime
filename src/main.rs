#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod types;

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
}
