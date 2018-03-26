//! The Rust Ohua Runtime Generator.
//!
//! This program generates a rust runtime for an [Ohua](https://github.com/ohua-dev) algorithm, which can be defined in an `ohuac` file.

mod types;
mod typecasts;
mod wrappers;
mod runtime_data;
mod modgen;

use serde_json;
use std::fs::{remove_dir_all, File, DirBuilder};
use std::io::{self, Write};
use std::path::Path;
use comp_errors::CodeGenerationError;
use self::types::{OhuaData, AlgorithmArguments};
use self::runtime_data::generate_runtime_data;


/// This function writes all static files to their respective locations,
/// returning an error when the write operation exitted unsuccessfully.
fn populate_static_files(path: String) -> io::Result<()> {
    let type_file = include_bytes!("templates/types.rs");
    File::create(path.clone() + "/types.rs")?.write_all(type_file)?;

    Ok(())
}

/// Runtime Generator
pub fn generate_ohua_runtime<P: AsRef<Path>>(dfg_path: P, typeinfo_path: P, mut output: String) -> Result<(), CodeGenerationError> {
    if let Err(err) = DirBuilder::new().recursive(true).create(output.as_str()) {
        return Err(CodeGenerationError::TargetDirNotCreated(err));
    }

    // ====== start the real work ======

    // read the data structures
    let dfg_file = File::open(dfg_path).unwrap();
    let ohua_data: OhuaData = serde_json::from_reader(dfg_file).unwrap();
    let typeinfo_file = File::open(typeinfo_path).unwrap();
    let type_data: AlgorithmArguments = serde_json::from_reader(typeinfo_file).unwrap();

    // check whether both mainArity and number of argument types match
    if ohua_data.mainArity as usize != type_data.argument_types.len() {
        return Err(CodeGenerationError::InconsistentMainArity);
    }

    // generate the module of the ohua runtime
    output += "/ohua_runtime";
    if let Err(err) = DirBuilder::new().create(output.as_str()) {
        // Remove the directory if it already exists.
        if err.kind() == io::ErrorKind::AlreadyExists {
            if let Err(delete_err) = remove_dir_all(output.as_str()) {
                return Err(CodeGenerationError::ModuleDirUndeletable(delete_err));
            } else {
                // recreate the empty dir after successful deletion
                if let Err(e) = DirBuilder::new().create(output.as_str()) {
                    return Err(CodeGenerationError::ModuleDirNotCreated(e));
                }
            }
        } else {
            return Err(CodeGenerationError::ModuleDirNotCreated(err));
        }
    }

    // TODO: Continue to rewrite into error type, write first `build.rs`, verify it works

    // populate the module with the static files
    if let Err(err) = populate_static_files(output.clone()) {
        return Err(CodeGenerationError::StaticPopulationFailed(err));
    }

    // generate all necessary type casts for the Arcs
    if let Err(err) = typecasts::generate_casts(&ohua_data.graph.operators, &type_data, (output.clone() + "/generictype.rs").as_str()) {
        return Err(CodeGenerationError::StaticPopulationFailed(err))
    }

    // generate wrapper functions for all operators
    let altered_ohuadata = match wrappers::generate_wrappers(ohua_data, &type_data.argument_types, (output.clone() + "/wrappers.rs").as_str()) {
        Ok(data) => data,
        Err(err) => {
            return Err(CodeGenerationError::WrapperGenerationFailed(err));
        }
    };

    if let Err(err) = modgen::generate_modfile(&type_data, (output.clone() + "/mod.rs").as_str()) {
        return Err(CodeGenerationError::ModfileGenerationFailed(err));
    }

    // write the runtime OhuaData structure
    if let Err(err) = generate_runtime_data(altered_ohuadata, (output + "/runtime.rs").as_str()) {
        return Err(CodeGenerationError::RuntimeDataCreationFailed(err));
    }

    Ok(())
}
