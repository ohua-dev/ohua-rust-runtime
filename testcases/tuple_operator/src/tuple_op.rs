pub fn calculate_string_length(input: String) -> usize {
    input.len()
}

pub fn splice_string(mut input: String) -> String {
    input += " foo";
    input
}

pub fn calculate_statistics(tup: (usize, String, usize)) -> f64 {
    tup.2 as f64 / tup.0 as f64
}
