// for `smap` test case

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

// for `smap_with_lambdas`
pub fn generate_value() -> i32 {
    4
}

pub fn generate_data() -> Vec<i32> {
    vec![2, 42, 7, 12, 185, 943, 375]
}

pub fn calculate(x: i32, y: i32) -> i32 {
    x * y
}
