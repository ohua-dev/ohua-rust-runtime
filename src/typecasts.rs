// maybe return Result?
pub fn generate_casts(types: Vec<&str>, output_file: &str ) {
    let generic_type_file = include_str!("templates/generictype.rs");
    let typecast = include_str!("templates/snippets/typecast.in");

    let mut wow: String = typecast.replace("{target_type}", "u32");
    wow.push_str(typecast.replace("{target_type}", "u16").as_ref());

    println!("{}{}", generic_type_file, wow);
}
