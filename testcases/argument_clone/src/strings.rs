pub fn gen_string(_: i32) -> String {
    String::from("Hello")
}

pub fn count_strings(a: String, b: String) -> i32 {
    (a.len() + b.len()) as i32
}
