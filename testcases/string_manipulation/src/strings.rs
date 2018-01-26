pub fn gen_s1(_: i32) -> String {
    String::from("Hello, ")
}

pub fn gen_s2(_: i32) -> String {
    String::from("World!")
}

pub fn append(s1: String, s2: String) -> String {
    s1 + s2.as_str()
}

pub fn duplicate(s: String) -> String {
    s.clone() + s.as_str()
}

pub fn count(dup: String) -> i32 {
    dup.len() as i32
}
