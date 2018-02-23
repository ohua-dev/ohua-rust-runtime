pub fn append_to_string(input_string: String, input_number: i32) -> String {
    input_string.replace("zero", input_number.to_string().as_str())
}

pub fn extend_string(intermediate: String) -> String {
    intermediate + " Or is it?"
}

pub fn output_values(input_number: i32, extended_string: String) -> (i32, String, usize) {
    let length = extended_string.len();
    (input_number, extended_string, length)
}
