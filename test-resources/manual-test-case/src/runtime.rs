use std::sync::mpsc::{Sender, Receiver};

// introduced because Rust does not allow Trait implementations for types that are defined elsewhere
#[derive(Debug)]
pub struct GenericType {}

impl From<Box<GenericType>> for Box<i32> {
    fn from(arg: Box<GenericType>) -> Self {
        // raw pointer cast. Quick and dirty. The joke is that the pointer cast
        // itself is ok as long as both types implement the `Sized` trait (I guess).
        // Box::from_raw is the unsafe kiddo
        unsafe { Box::from_raw(Box::into_raw(arg) as *mut i32) }
    }
}

impl From<Box<i32>> for Box<GenericType> {
    fn from(arg: Box<i32>) -> Self {
        unsafe { Box::from_raw(Box::into_raw(arg) as *mut GenericType) }
    }
}

// TODO: Maybe move things from above to separate file (because it's dynamic)?

// maybe extend this with a name?
//#[derive(Debug)]
pub struct OhuaOperator {
    pub input: Vec<Receiver<Box<GenericType>>>,
    pub output: Vec<Sender<Box<GenericType>>>,
    pub func: Box<fn(Vec<Box<GenericType>>) -> Vec<Box<GenericType>>>,
}
