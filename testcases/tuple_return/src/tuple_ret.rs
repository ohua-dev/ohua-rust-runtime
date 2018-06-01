pub fn calculate_string_length(input: String) -> usize {
    input.len()
}

pub fn splice_string(mut input: String) -> String {
    input += " foo";
    input
}
