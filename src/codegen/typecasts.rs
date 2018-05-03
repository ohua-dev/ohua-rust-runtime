//! Typecasting generation for the `GenericType` type
use ohua_types::{Operator, OperatorType};
use type_extract::TypeKnowledgeBase;

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

/// Retrieves the argument types for a given function from the type knowledge base
fn get_argument_types(func: &OperatorType, lookup: &TypeKnowledgeBase) -> Vec<String> {
    // first off, find the function in the lookup table
    for funcinfo in &lookup.functions {
        if funcinfo.name == func.qbName && funcinfo.namespace == func.qbNamespace {
            // when we've found our function, collect all types
            let mut types = Vec::with_capacity(funcinfo.arguments.len() + 1);
            for ty in &funcinfo.arguments {
                types.push(ty.name.clone());
            }
            types.push(funcinfo.return_val.name.clone());
            return types;
        }
    }

    // if we've found nothing this is an error
    panic!(
        "The function {} has not been found in the type lookup table.",
        func.function_name()
    );
}

/// Retrieves and collects all types we have to generate casts for.
///
/// The types are retrieved either from the `type-dump` file or from the function headers of the stateful functions.
/// The returned result is forwarded from the `generate_for` function.
pub fn generate_casts(
    operators: &Vec<Operator>,
    typeinfo: &TypeKnowledgeBase,
    target_file: &str,
) -> Result<()> {
    let mut used_types: HashSet<String> = HashSet::new();

    // also make use of the argument types provided from the `type_dump` file
    for arg in &typeinfo.algo_io.argument_types {
        used_types.insert(arg.clone().replace(" ", ""));
    }
    used_types.insert(typeinfo.algo_io.return_type.clone().replace(" ", ""));

    for op in operators {
        // retrieve all argument types for the operator
        for occuring_type in get_argument_types(&op.operatorType, &typeinfo) {
            used_types.insert(occuring_type.replace(" ", ""));
        }
    }

    generate_for(used_types, target_file)
}
