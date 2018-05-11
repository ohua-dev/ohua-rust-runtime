//! Module File generator
use type_extract::TypeKnowledgeBase;
use std::io::{Result, Write};
use std::fs::File;

/// This function creates the `mod.rs` file that contains the main entry point into the algorithm.
///
/// By design, the module file only contains templating placeholders in the entry function header
/// and the return point, where the provided type signatures of the algorithm are inserted.
/// Only produces an IO error when unable to open and write to a file.
pub fn generate_modfile(ty_knowledge: &TypeKnowledgeBase, target_path: &str) -> Result<()> {
    let mod_file = include_str!("templates/mod.rs");
    let input_send = include_str!("templates/snippets/input_send.in");

    // check if any arguments to the function require an import
    let mut imports = String::new();
    for imp in ty_knowledge.find_imports_for_algo_io() {
        imports += &format!("use {};\n", imp);
    }

    // simultaneously construct the string that contains the function header args and
    // the initial argument handovers to the ohua runtime
    let mut function_arguments = String::new();
    let mut inputs = String::new();
    for (ind, typename) in ty_knowledge.algo_io.argument_types.iter().enumerate() {
        function_arguments +=
            format!("arg{n}: {typename}, ", n = ind, typename = typename).as_str();
        inputs += input_send.replace("{n}", &ind.to_string()).as_str();
    }

    let mut module = String::from(mod_file).replace("{ty_imports}", imports.as_str());
    module = module.replace("{input_args}", function_arguments.as_str());
    module = module.replace("{return_type}", ty_knowledge.algo_io.return_type.as_str());
    module = module.replace("{send_input}", inputs.as_str());

    File::create(target_path)?.write_fmt(format_args!("{}", module))
}
