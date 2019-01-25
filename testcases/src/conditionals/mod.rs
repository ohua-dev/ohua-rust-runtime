mod iftest;
mod calculations;

use ohua_codegen::ohua;

// same var as input for both branches
#[test]
fn test_basic_0() {
    #[ohua]
    let result = conditionals::algorithms::if_test_base_0();
    assert!(result == "executed: yes");
}

// no var as input for branches
#[test]
fn test_basic_1() {
    #[ohua]
    let result = conditionals::algorithms::if_test_base_1();
    assert!(result == "executed: yes");
}

#[test]
fn lambda_in_if() {
    #[ohua]
    let x = conditionals::algorithms::lambda_test();

    assert!(x == 8);
}

// FIXME: Frozen until closure of ohua-dev/ohua-code#28
// #[test]
// fn envarcs_with_if() {
//     let inp = String::from("executed: ");

//     #[ohua]
//     let result = conditionals::algorithms::if_envarcs(true, inp);

//     assert!(result == "executed: yes");
// }

// FIXME: Frozen until closure of ohua-dev/ohua-code#29
// #[test]
// fn if_in_if() {
//     #[ohua]
//     let result = conditionals::algorithms::ifinif();
//
//     assert!(result == "executed: no");
// }
