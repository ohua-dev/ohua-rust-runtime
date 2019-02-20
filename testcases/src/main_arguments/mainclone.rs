#![allow(dead_code)]

pub fn double(arg: i32) -> i32 {
    println!("Input: {}", arg);
    arg * 2
}

pub fn triple(arg: i32) -> i32 {
    println!("Intermediate: {}", arg);
    arg * 3
}

pub fn concat(mut arg1: String, arg2: String) -> String {
    arg1 += &arg2;
    arg1
}

pub fn append_foo(mut arg: String) -> String {
    arg += " foo";
    arg
}

pub fn expand_content(a: String) -> String {
    a + " over the hill"
}

pub fn splice_message(a: String, b: String) -> String {
    a + &b
}
