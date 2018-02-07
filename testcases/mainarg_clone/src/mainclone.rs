pub fn calc(arg1: i32, arg2: i32) -> i32 {
    println!("Input: {} and {}", arg1, arg2);
    arg1 + arg2
}

pub fn double(arg: i32) -> i32 {
    println!("Intermediate: {}", arg);
    arg * 2
}
