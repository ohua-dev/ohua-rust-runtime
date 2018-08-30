mod house;
mod ohua_runtime;

use house::{EmotionalState, House, Human};

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

    let new_house = ohua_runtime::ohua_main(home, String::from("1323 2nd Street"), humans);

    println!("We now got {} inhabitants!", new_house.inhabitants.len());
}
