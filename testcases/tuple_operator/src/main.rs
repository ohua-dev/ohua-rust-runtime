mod ohua_runtime;
mod tuple_op;

fn main() {
    let input = String::from("This is a test string.");
    let (spliced, stat) = ohua_runtime::ohua_main(input);

    println!("String \"{}\" grew/shrinked by factor: {}", spliced, stat);
}
