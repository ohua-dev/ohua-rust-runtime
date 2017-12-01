// this file is NOT part of the project itsef, it's provided by the dev
pub fn calc(arg: i32) -> i32 {
    println!("Input: {}", arg);
    arg + arg
}

pub fn world(arg: i32) -> i32 {
    println!("Intermediate: {}", arg);
    arg * 2
}
