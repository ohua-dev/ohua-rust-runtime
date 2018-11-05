pub fn generate_string() -> String {
    "This string contains the number ".into()
}

pub fn recv_number() -> i32 {
    42
} 

pub fn combine(s: String, num: i32) -> String {
    format!("{}{}", s, num)
}

pub fn printout(s: String) {
    println!("{}", s);
}
