#![feature(proc_macro_hygiene, fnbox, custom_attribute)]

extern crate ohua_codegen;
extern crate ohua_runtime;

mod iftest;

// fn main() {
//     #[ohua]
//     let result = if_test();
//
//     assert!(result == "executed: yes");
// }

#[cfg(test)]
mod tests {
    use ohua_codegen::ohua;

    // same var as input for both branches
    #[test]
    fn test_basic_0() {
        #[ohua]
        let result = if_test();
        assert!(result == "executed: yes");
    }

    // no var as input for branches
    #[test]
    fn test_basic_1() {
        #[ohua]
        let result = if_test_base_1();
        assert!(result == "executed: yes");
    }
}
