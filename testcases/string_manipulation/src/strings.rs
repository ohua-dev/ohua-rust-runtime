pub fn extend_string1(input: String) -> String {
    format!("From `{}` to this.", input)
}

pub fn extend_string2(input: String) -> String {
    format!(" A big `{}`-extension!", input)
}

pub fn append(s1: String, s2: String) -> String {
    s1 + s2.as_str()
}

pub fn duplicate(s: String) -> String {
    s.clone() + s.as_str()
}

pub fn count(dup: String) -> usize {
    dup.len()
}
