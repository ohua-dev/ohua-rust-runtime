use hello;
use runtime::*;

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
