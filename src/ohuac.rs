//! Wrappers around the `ohuac` binary

use std::path::PathBuf;
use std::process::Command;
use std::io::ErrorKind;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Debug)]
pub struct OhuaProduction {
    pub src: PathBuf,
    pub ohuao: PathBuf,
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

// TODO: Maybe return Hashmap to remove any dupes?
pub fn generate_dfgs(mut sources: Vec<PathBuf>, target_dir: PathBuf) -> Vec<OhuaProduction> {
    let mut processed_algos: Vec<OhuaProduction> = Vec::new();

    // run the type dump and object file creation for all ohuac files found
    for src in sources.drain(..) {
        let mut dfg_file = target_dir.clone();
        dfg_file.push(hashed_filename(&src));
        dfg_file.set_extension("ohuao");

        // build the `ohuao` file
        let dfg_output = match Command::new("ohuac")
            .args(&[
                "build",
                src.to_str().unwrap(),
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
                src.to_str().unwrap(),
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
                src.to_str().unwrap(),
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
            println!("[Phase 1] Processed file {}", src.to_str().unwrap());
        } else {
            panic!(
                "[Phase 1] `ohuac` failed to process file {}: {}",
                src.to_str().unwrap(),
                String::from_utf8(type_output.stderr).unwrap()
            );
        }

        // if everything went smoothly, book the file in
        processed_algos.push(OhuaProduction {
            src: src,
            ohuao: dfg_file,
            typedump: type_file,
        });
    }

    processed_algos
}
