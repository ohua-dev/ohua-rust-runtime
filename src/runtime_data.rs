use types::OhuaData;

pub fn generate_runtime_data(ohuadata: &OhuaData, target_file: &str) {
    let template = include_str!("templates/runtime.rs");

    println!("{}", template.replace("{dump}", format!("{}", ohuadata).as_str()));
}
