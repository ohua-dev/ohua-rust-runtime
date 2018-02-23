mod ohua_runtime;
mod tuples;

fn main() {
    let (num, text, num2) = ohua_runtime::ohua_main(String::from("The number is zero."), 42);

    println!("Number: {} (84), string: {}, number 2: {}", num, text, num2)
}
