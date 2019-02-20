mod house;
mod mainclone;

use ohua_codegen::ohua;

#[test]
fn main_arguments() {
    #[ohua]
    main_arguments::algorithms::mainargs(15);
}

#[test]
fn main_arguments_clone() {
    let input = String::from("The quick brown fox");

    #[ohua]
    main_arguments::algorithms::mainarg_cloning(input);
}

#[test]
fn main_arguments_across_ops() {
    let text = String::from("the quick brown fox jumped");

    #[ohua]
    let result = main_arguments::algorithms::reuse(text);

    println!("Computation result: {}", result);
}

#[test]
fn custom_types_via_envarcs() {
    use self::house::{EmotionalState, House, Human};

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
    let new_house =
        main_arguments::algorithms::custom_types(home, String::from("1323 2nd Street"), humans);

    println!("We now got {} inhabitants!", new_house.inhabitants.len());
}
