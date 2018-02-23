mod strings;
mod ohua_runtime;

fn main() {
    let output = ohua_runtime::ohua_main(String::from("input"));
    println!("The length of the final string ('From `input` to this. A big `input`-extension!') is {} (should be 46*2=92).", output);
}
