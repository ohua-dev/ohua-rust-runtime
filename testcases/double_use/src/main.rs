mod hello;
mod ohua_runtime;

fn main() {
    let number = 12;
    let result: i32 = ohua_runtime::ohua_main(number);

    println!("Result of (12 + 12 + 12) * 2: {}", result);
}
