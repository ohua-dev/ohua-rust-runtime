#![feature(proc_macro_hygiene, fnbox)]

use ohua_codegen::ohua;

mod io_stuff;

fn complete_string(s: String) -> String {
    s + " giant spiders"
}

fn main() {
    let inputs: Vec<String> = vec![
        "I hate".into(),
        "Why are there everywhere".into(),
        "there is a huge pile of".into(),
    ];

    #[ohua]
    let x = algo::string_manipulation(inputs);

    assert!(
        x == vec![
            "I hate giant spiders",
            "Why are there everywhere giant spiders",
            "there is a huge pile of giant spiders"
        ]
    );
}
