// pub fn add(a: u32) -> u32 {
//     a + 42
// }

pub fn produce() -> u32 {
    println!("producing");
    42
}

pub fn consume(_v: u32) {
    println!("consuming");
}
