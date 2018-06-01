mod ohua_runtime;
mod tuple_ret;

fn main() {
    let input = String::from("This is a test string.");
    let (old, spliced, new) = ohua_runtime::ohua_main(input);

    assert!(old < new);
    println!(
        "Old/new string length: {}/{} -- String: \"{}\"",
        old, new, spliced
    );
}
