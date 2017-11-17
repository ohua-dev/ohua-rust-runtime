#![allow(unused_variables)]

mod types;
mod hello;
use types::*;

use std::thread;
use std::sync::mpsc;
// use std::mem::transmute;

fn generate_dummy() -> OhuaData {
    OhuaData {
        graph: DFGraph {
            operators: vec![Operator {
                                operatorId: 1,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("calc"),
                                    func: Box::new(hello::calc),
                                },
                            },
                            Operator {
                                operatorId: 2,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("world"),
                                    func: Box::new(hello::world),
                                },
                            }],
            arcs: vec![Arc {
                           target: ArcIdentifier {
                               operator: 1,
                               index: 0,
                           },
                           source: ArcSource {
                               s_type: String::from("env"),
                               val: ValueType::EnvironmentVal(0),
                           },
                       },
                       Arc {
                           target: ArcIdentifier {
                               operator: 2,
                               index: 0,
                           },
                           source: ArcSource {
                               s_type: String::from("local"),
                               val: ValueType::LocalVal(ArcIdentifier {
                                                            operator: 1,
                                                            index: -1,
                                                        }),
                           },
                       }],
        },
        mainArity: 0,
        sfDependencies: vec![SfDependency {
                                 qbNamespace: vec![String::from("hello")],
                                 qbName: String::from("calc"),
                             },
                             SfDependency {
                                 qbNamespace: vec![String::from("hello")],
                                 qbName: String::from("world"),
                             }],
    }
}

// introduced because Rust does not allow Trait implementations for types that are defined elsewhere
#[derive(Debug)]
struct GenericType {}


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

fn calc_wrapped(mut args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    let arg1 = Box::from(args.pop().unwrap());

    let res = Box::new(hello::calc(*arg1));

    vec![Box::from(res)]
}

fn world_wrapped(mut args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    let arg1 = Box::from(args.pop().unwrap());

    let res = Box::new(hello::world(*arg1));

    vec![Box::from(res)]
}


fn main() {
    // let's just assume this was somehow extracted/generated
    let deserialized_data = generate_dummy();

    // channel creation
    let (insertion_point, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();
    let (s3, output_port) = mpsc::channel();

    // thread spawning
    thread::spawn(move || {
                      // receive
                      let args = vec![r1.recv().unwrap()];

                      // call & send
                      let mut results = calc_wrapped(args);
                      s2.send(results.pop().unwrap()).unwrap();
                  });

    thread::spawn(move || {
                      // receive
                      let args = vec![r2.recv().unwrap()];

                      // call & send
                      let mut results = world_wrapped(args);
                      s3.send(results.pop().unwrap()).unwrap();
                  });

    // providing input to the DFG
    insertion_point.send(Box::from(Box::new(3))).unwrap();

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
