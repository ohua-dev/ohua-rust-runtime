use hello;
use generictype::*;

pub fn calc_wrapped(mut args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    // this function stays always almost the same. Only name, function call and argument extraction have to be generated
    let arg1 = Box::from(args.pop().unwrap());

    let res = Box::new(hello::calc(*arg1));

    vec![Box::from(res)]
}

pub fn world_wrapped(mut args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    let arg1 = Box::from(args.pop().unwrap());

    let res = Box::new(hello::world(*arg1));

    vec![Box::from(res)]
}


// TODO: needs to be generated additionally
// these functions will provide the arguments the main function arguments.
pub fn mainarg0(args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    vec![Box::from(Box::new(3))]
}
