#![allow(dead_code)]

pub fn produce() -> u32 {
    println!("producing");
    42
}

pub fn consume(_v: u32) {
    println!("consuming");
}
