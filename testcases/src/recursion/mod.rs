mod sfs;

use ohua_codegen::ohua;

#[test]
fn test_simple_recursion() {
    #[ohua]
    let result = recursion::algorithms::simple_recur();
    assert!(result == 0);
}

#[test]
fn test_simple_tl_recursion() {
    #[ohua]
    let result = recursion::algorithms::simple_tl_recur();
    assert!(result == 0);
}
