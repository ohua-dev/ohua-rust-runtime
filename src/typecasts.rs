use types::Operator;

use std::collections::HashSet;
use std::fs::File;
use std::io::{Result, Write};

// maybe return Result?
fn generate_for(types: HashSet<String>, target_file: &str) -> Result<()> {
    let generic_type_file = include_str!("templates/generictype.rs");
    let typecast = include_str!("templates/snippets/typecast.in");

    let mut typecast_file = String::new();
    
    for datatype in types {
        typecast_file.push_str(typecast.replace("{target_type}", datatype.as_str()).as_str());
    }

    File::create(target_file)?.write_fmt(format_args!("{}{}", generic_type_file, typecast_file))
}

fn get_argument_types(fn_name: String) -> Vec<String> {
    match fn_name.as_str() {
        "hello::calc" => vec![String::from("i32")],
        "hello::world" => vec![String::from("i32")],
        _ => vec![]
    }
}

pub fn generate_casts(operators: &Vec<Operator>, target_file: &str) -> Result<()> {
    let mut used_types: HashSet<String> = HashSet::new();

    for op in operators {
        let fn_name = op.operatorType
                        .qbNamespace
                        .iter()
                        .fold(String::new(), |acc, ref x|
                            acc.to_owned() + &x + "::"
                        ) + op.operatorType.qbName.as_str();

        for occuring_type in get_argument_types(fn_name) {
            used_types.insert(occuring_type);
        }
    }

    generate_for(used_types, target_file)
}
