//! Module File generator
use super::types::AlgorithmArguments;
use std::io::{Result, Write};
use std::fs::File;

/// This function creates the `mod.rs` file that contains the main entry point into the algorithm.
///
/// By design, the module file only contains templating placeholders in the entry function header
/// and the return point, where the provided type signatures of the algorithm are inserted.
/// Only produces an IO error when unable to open and write to a file.
pub fn generate_modfile(mainarg_types: &AlgorithmArguments, target_path: &str) -> Result<()> {
    let mod_file = include_str!("templates/mod.rs");
    let input_send = include_str!("templates/snippets/input_send.in");

    // simultaneously construct the string that contains the function header args and
    // the initial argument handovers to the ohua runtime
    let mut function_arguments = String::new();
    let mut inputs = String::new();
    for (ind, typename) in mainarg_types.argument_types.iter().enumerate() {
        function_arguments +=
            format!("arg{n}: {typename}, ", n = ind, typename = typename).as_str();
        inputs += input_send.replace("{n}", &ind.to_string()).as_str();
    }

    let mut module = String::from(mod_file).replace("{input_args}", function_arguments.as_str());
    module = module.replace("{return_type}", mainarg_types.return_type.as_str());
    module = module.replace("{send_input}", inputs.as_str());

    File::create(target_path)?.write_fmt(format_args!("{}", module))
}
