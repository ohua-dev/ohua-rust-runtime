pub fn get_cond_result() -> bool {
    true
}

pub fn get_input() -> String {
    String::from("executed: ")
}

pub fn id<T: Clone + Send>(x: T) -> T {
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

// for if_in_if
pub fn get_ctrl_input() -> bool {
    true
}

pub fn get_another_ctrl_input() -> bool {
    false
}

// for smap_in_if

fn get_vec_input() -> Vec<i32> {
    vec![2, 4, 6, 8, 10, 12]
}

fn times_2(num: i32) -> i32 {
    num * 2
}
