//! Typecasting generation for the `GenericType` type
use types::{AlgorithmArguments, Operator};

use std::collections::HashSet;
use std::fs::File;
use std::io::{Result, Write};

/// Generates a bidirectional type cast for every type from the given hash set.
///
/// Returns an IO Error when opening or writing to a file fails.
fn generate_for(types: HashSet<String>, target_file: &str) -> Result<()> {
    let generic_type_file = include_str!("templates/generictype.rs");
    let typecast = include_str!("templates/snippets/typecast.in");

    let mut typecast_file = String::new();

    for datatype in types {
        typecast_file.push_str(
            typecast
                .replace("{target_type}", datatype.as_str())
                .as_str(),
        );
    }

    File::create(target_file)?.write_fmt(format_args!("{}{}", generic_type_file, typecast_file))
}

/// Currently a dummy function to "extract" the types. Has to be extended everytime with the new cases.
fn get_argument_types(fn_name: String) -> Vec<String> {
    match fn_name.as_str() {
        "hello::calc" => vec![String::from("i32")],
        "hello::world" => vec![String::from("i32")],
        "strings::gen_string" => vec![String::from("i32")],
        "strings::count_strings" => vec![String::from("String")],
        "strings::extend_string1" => vec![String::from("String")],
        "strings::extend_string2" => vec![String::from("String")],
        "strings::append" => vec![String::from("String")],
        "strings::duplicate" => vec![String::from("String")],
        "strings::count" => vec![String::from("String"), String::from("usize")],
        "mainclone::calc" => vec![String::from("i32")],
        "mainclone::double" => vec![String::from("i32")],
        "tuples::append_to_string" => vec![String::from("String"), String::from("i32")],
        "tuples::extend_string" => vec![String::from("String")],
        "tuples::output_values" => vec![String::from("i32"), String::from("String"), String::from("(i32, String, usize)")],
        _ => vec![],
    }
}

/// Retrieves and collects all types we have to generate casts for.
///
/// The types are retrieved either from the `type-dump` file or from the function headers of the stateful functions.
/// The returned result is forwarded from the `generate_for` function.
pub fn generate_casts(
    operators: &Vec<Operator>,
    algo_args: &AlgorithmArguments,
    target_file: &str,
) -> Result<()> {
    let mut used_types: HashSet<String> = HashSet::new();

    // also make use of the argument types provided from the `type_dump` file
    for arg in &algo_args.argument_types {
        used_types.insert(arg.clone());
    }
    used_types.insert(algo_args.return_type.clone());

    for op in operators {
        let fn_name = op.operatorType
            .qbNamespace
            .iter()
            .fold(String::new(), |acc, ref x| acc.to_owned() + &x + "::")
            + op.operatorType.qbName.as_str();

        for occuring_type in get_argument_types(fn_name) {
            used_types.insert(occuring_type);
        }
    }

    generate_for(used_types, target_file)
}
