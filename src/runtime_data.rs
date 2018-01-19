use types::OhuaData;

use std::fs::File;
use std::io::{Result, Write};

pub fn generate_runtime_data(ohuadata: &OhuaData, target_file: &str) -> Result<()> {
    let template = include_str!("templates/runtime.rs");

    let filled = template.replace("{dump}", format!("{}", ohuadata).as_str());
    File::create(target_file)?.write_fmt(format_args!("{}", filled))
}
