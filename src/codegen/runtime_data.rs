//! Runtime `OhuaData` generator
use ohua_types::OhuaData;

use std::fs::File;
use std::io::{Result, Write};

/// This function dumps the `OhuaData` structure to the `runtime.rs` file.
///
/// Intentionally takes ownership of the OhuaData struct as this should be the last
/// operation performed to avoid data inconsistencies.
/// If a filesystem problem occurs, the corresponding error will be returned.
pub fn generate_runtime_data(ohuadata: OhuaData, target_file: &str) -> Result<()> {
    let template = include_str!("templates/runtime.rs");

    let filled = template.replace("{dump}", &ohuadata.to_string());
    File::create(target_file)?.write_fmt(format_args!("{}", filled))
}
