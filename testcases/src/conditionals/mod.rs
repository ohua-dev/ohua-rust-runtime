mod calculations;
mod iftest;

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

// FIXME: Frozen until closure of ohua-dev/ohua-core#28
#[test]
fn envarcs_with_if() {
    unimplemented!("Frozen until closure of ohua-dev/ohua-core#28")
    // let inp = String::from("executed: ");
    //
    // #[ohua]
    // let result = conditionals::algorithms::if_envarcs(true, inp);
    //
    // assert!(result == "executed: yes");
}

#[test]
fn if_in_if() {
    unimplemented!("Generated `ctrl` Operator has too many out-arcs supplied as inputs")
    // #[ohua]
    // let result = conditionals::algorithms::ifinif();
    //
    // assert!(result == "executed: no");
}

#[test]
fn smap_in_if_no_passthrough() {
    // unimplemented!("Frozen until closure of ohua-dev/ohua-core#29")
    #[ohua]
    let res = conditionals::algorithms::smap_in_if_no_passthrough();

    assert!(res == vec![4, 8, 12, 16, 20, 24]);
}

#[test]
fn smap_in_if_passthrough() {
    // unimplemented!("Frozen until closure of ohua-dev/ohua-core#29")
    #[ohua]
    let res = conditionals::algorithms::smap_in_if_passthrough();

    assert!(res == vec![4, 8, 12, 16, 20, 24]);
}
