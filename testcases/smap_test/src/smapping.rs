pub fn gen_input() -> Vec<String> {
    vec![
        "I hate".into(),
        "Why are there everywhere".into(),
        "there is a huge pile of".into(),
    ]
}

pub fn printout(s: String) {
    println!("{}", s);
}

pub fn splice(s: String) -> String {
    s + " giant spiders"
}
