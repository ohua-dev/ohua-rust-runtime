// maybe return Result?
pub fn generate_casts(types: Vec<&str>, output_file: &str) {
    let generic_type_file = include_str!("templates/generictype.rs");
    let typecast = include_str!("templates/snippets/typecast.in");

    let mut typecast_file = String::new();
    
    for datatype in types {
        typecast_file.push_str(typecast.replace("{target_type}", datatype).as_ref());
    }

    println!("{}{}", generic_type_file, typecast_file);
}
