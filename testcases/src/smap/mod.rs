mod smap_fns;

use ohua_codegen::ohua;

#[test]
fn smap() {
    #[ohua]
    let x = smap::algorithms::smap_test();

    assert!(
        x == vec![
            "I hate giant spiders",
            "Why are there everywhere giant spiders",
            "there is a huge pile of giant spiders"
        ]
    );
}

#[test]
fn smap_with_lambdas() {
    #[ohua]
    let x = smap::algorithms::lambda_test();

    assert!(x == vec![8, 168, 28, 48, 740, 3772, 1500]);
}

#[test]
fn smap_with_envarc_input() {
    let inputs: Vec<String> = vec![
        "I hate".into(),
        "Why are there everywhere".into(),
        "there is a huge pile of".into(),
    ];

    #[ohua]
    let x = smap::algorithms::smap_env_test(inputs);

    assert!(
        x == vec![
            "I hate giant spiders",
            "Why are there everywhere giant spiders",
            "there is a huge pile of giant spiders"
        ]
    );
}

// FIXME: Frozen until closure of ohua-dev/ohua-code#28
// #[test]
// fn smap_with_envarc_in_loop() {
//     #[ohua]
//     let x = smap::algorithms::smap_env_in_loop(String::from(" giant spiders"));
//
//     assert!(
//         x == vec![
//             "I hate giant spiders",
//             "Why are there everywhere giant spiders",
//             "there is a huge pile of giant spiders"
//         ]
//     );
// }

// #[test]
// fn smap_seq_test() {
//     #[ohua]
//     let x = smap::algorithms::seq_test();
//
//     assert!(x == vec![4, 4, 4, 4, 4, 4, 4]);
// }

// #[test]
// fn if_in_smap() {
//     #[ohua]
//     let x = smap::algorithms::if_in_smap();
//
//     assert!(x == vec![8, 168, 0, 48, 0, 0, 0]);
// }
