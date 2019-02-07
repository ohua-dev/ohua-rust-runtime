
pub fn lt<T : std::cmp::PartialOrd>(a:T, b:T) -> bool {
    a < b
}

pub fn plus<T : std::ops::Add>(a:T, b:T) -> <T as std::ops::Add>::Output {
    a + b
}
