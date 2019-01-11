#![allow(dead_code)]

// for the `argument clone` testcase

pub fn gen_string() -> String {
    String::from("Hello")
}

pub fn count_strings(a: String, b: String) -> i32 {
    let result = (a.len() + b.len()) as i32;
    println!("{}", result);
    result
}

// for the `lambda` testcase

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
