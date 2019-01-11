mod produce_consume;
mod strings;
mod house;

use ohua_codegen::ohua;

#[test]
fn ohua_macro_test() {
    #[ohua]
    general::algorithms::produce_consume();
}

#[test]
fn run_multiple_algos() {
    #[ohua]
    general::algorithms::mult_algos::foobar();

    #[ohua]
    general::algorithms::mult_algos::something::different();
}

#[test]
fn automatic_arg_cloning() {
    #[ohua]
    general::algorithms::argument_clone();
}

#[test]
fn custom_types() {
    #[ohua]
    general::algorithms::custom_types();
}

#[test]
fn lambdas() {
    #[ohua]
    general::algorithms::lambdas();
}
