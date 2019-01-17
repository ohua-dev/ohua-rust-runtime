pub fn get_cond_result() -> bool {
    true
}

pub fn get_input() -> String {
    String::from("executed: ")
}

pub fn id<T:Clone + Send>(x:T) -> T {
    x
}

pub fn executed_yes() -> String {
    String::from("executed: yes")
}

pub fn executed_no() -> String {
    String::from("executed: no")
}

pub fn modify_string_positive(a: String) -> String {
    a + "yes"
}

pub fn modify_string_negative(a: String) -> String {
    a + "no"
}
