#![feature(proc_macro_hygiene, fnbox)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod house;

use house::{EmotionalState, House, Human};
use ohua_codegen::ohua;

fn main() {
    let humans = vec![
        Human {
            name: String::from("John Doe"),
            emotional_state: EmotionalState::Content,
            age: 42,
        },
        Human {
            name: String::from("Jane Doe"),
            emotional_state: EmotionalState::Happy,
            age: 40,
        },
        Human {
            name: String::from("Ayn Rand"),
            emotional_state: EmotionalState::Sad,
            age: 28,
        },
    ];
    let home = House {
        rooms: 6,
        inhabitants: vec![Human {
            name: String::from("江戸川 コナン"),
            emotional_state: EmotionalState::Content,
            age: 17,
        }],
        address: String::from("3687 1st Ave"),
    };

    #[ohua]
    let new_house = custom_types(home, String::from("1323 2nd Street"), humans);

    println!("We now got {} inhabitants!", new_house.inhabitants.len());
}
