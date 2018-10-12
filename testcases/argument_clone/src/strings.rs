pub fn gen_string() -> String {
    String::from("Hello")
}

pub fn count_strings(a: String, b: String) -> i32 {
    let result = (a.len() + b.len()) as i32;
    println!("{}", result);
    result
}
