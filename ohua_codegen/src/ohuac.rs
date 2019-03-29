//! Wrappers around the `ohuac` binary

use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct OhuaProduction {
    /// The algorithm's name
    pub name: String,
    /// Path where the source `ohua` algorithm definition file is stored
    pub src: PathBuf,
    /// Path where the compiled algorithm object file lives _(points to a temp directory!)_
    pub ohuao: PathBuf,
    /// Path where the typedump file generated by `ohuac` lives _(points to a temp directory!)_
    pub typedump: PathBuf,
}

/// Creates a hashed filename, based on the original path's hash
fn hashed_filename(input_name: &PathBuf) -> String {
    let mut output_name: String = input_name.file_stem().unwrap().to_str().unwrap().into();

    let mut hasher = DefaultHasher::new();
    input_name.hash(&mut hasher);
    output_name += &hasher.finish().to_string();

    output_name
}

pub fn generate_dfg(source: PathBuf, target_dir: PathBuf) -> OhuaProduction {
    // run the type dump and object file creation for the ohuac file in question
    let mut dfg_file = target_dir.clone();
    dfg_file.push(hashed_filename(&source));
    dfg_file.set_extension("ohuao");

    // build the `ohuao` file
    let dfg_output = match Command::new("ohuac")
        .args(&[
            "build",
            "-f",
            "tail-recursion",
            source.to_str().unwrap(),
            "--output",
            dfg_file.to_str().unwrap(),
        ])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                panic!("The `ohuac` executable could not be found. Please make sure to have `ohuac` installed and in your $PATH.");
            } else {
                panic!("Unable to spawn `ohuac`. {}", e.description());
            }
        }
    };

    if !dfg_output.status.success() {
        panic!(
            "[Phase 1] `ohuac` failed to process file {}: {}",
            source.to_str().unwrap(),
            String::from_utf8(dfg_output.stderr).unwrap()
        );
    }

    // build the `type-dump` file
    let mut type_file = dfg_file.clone();
    type_file.set_extension("type-dump");
    let type_output = match Command::new("ohuac")
        .args(&[
            "dump-main-type",
            "rust",
            source.to_str().unwrap(),
            "--output",
            type_file.to_str().unwrap(),
        ])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                panic!("The `ohuac` executable could not be found. Please make sure to have `ohuac` installed and in your $PATH.");
            } else {
                panic!("Unable to spawn `ohuac`. {}", e.description());
            }
        }
    };

    if type_output.status.success() {
        println!("[Phase 1] Processed file {}", source.to_str().unwrap());
    } else {
        panic!(
            "[Phase 1] `ohuac` failed to process file {}: {}",
            source.to_str().unwrap(),
            String::from_utf8(type_output.stderr).unwrap()
        );
    }

    // if everything went smoothly, return the files neatly packaged
    OhuaProduction {
        name: source.file_stem().unwrap().to_str().unwrap().into(),
        src: source,
        ohuao: dfg_file,
        typedump: type_file,
    }
}
