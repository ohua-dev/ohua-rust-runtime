use std::fs::{DirBuilder, File};
use std::io::{Result, Write};

use ohua_types::OhuaData;

pub fn generate_operators(_op: &mut OhuaData, output_base: String) -> Result<()> {
    // first off, generate the `operators` submodule
    let output = output_base + "/operators";
    DirBuilder::new().create(output.as_str())?;

    // generate the `mod.rs` file
    let modrs = include_str!("templates/mod.rs");
    File::create(output.clone() + "/mod.rs")?.write_fmt(format_args!("{skel}", skel = modrs))?;

    Ok(())
}

