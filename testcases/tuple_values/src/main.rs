mod ohua_runtime;
mod tuples;

fn main() {
    let (num, text, num2) = ohua_runtime::ohua_main(42, String::from("The number is zero."));

    println!("Number: {} (84), string: \"{}\", number 2: {}", num, text, num2)
}
