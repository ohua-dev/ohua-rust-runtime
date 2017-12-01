#![allow(unused_variables)]

mod types;
mod hello;
mod runtime;
use types::*;
use runtime::*;

use std::thread;
use std::sync::mpsc;


fn generate_ohuadata() -> OhuaData {
    // CAVEATS: make sure the operator vector is sorted by operatorId!
    OhuaData {
        graph: DFGraph {
            operators: vec![Operator {
                                operatorId: 1,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("calc"),
                                    func: Box::new(calc_wrapped),
                                },
                            },
                            Operator {
                                operatorId: 2,
                                operatorType: OperatorType {
                                    qbNamespace: vec![String::from("hello")],
                                    qbName: String::from("world"),
                                    func: Box::new(world_wrapped),
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


fn calc_wrapped(mut args: Vec<Box<GenericType>>) -> Vec<Box<GenericType>> {
    // this function stays always almost the same. Only name, function call and argument extraction have to be generated
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
    // let's just assume this function will be generated
    let runtime_data = generate_ohuadata();

    // instantiate the operator(s)
    let mut operators: Vec<OhuaOperator> = Vec::new();

    // statically fill the operator struct
    for op in runtime_data.graph.operators {
        operators.push(OhuaOperator {
                           input: vec![],
                           output: vec![],
                           func: op.operatorType.func,
                       })
    }

    // for arc in runtime_data.graph.arcs {}

    // channel creation
    // TODO: rework
    let (insertion_point, r1) = mpsc::channel();
    operators[0].input.push(r1);
    let (s2, r2) = mpsc::channel();
    operators[0].output.push(s2);
    operators[1].input.push(r2);
    let (s3, output_port) = mpsc::channel();
    operators[1].output.push(s3);

    // thread spawning -- static
    for op in operators.drain(..) {
        thread::spawn(move || {
            // receive
            let mut args = vec![];
            for recv in op.input {
                args.push(recv.recv().unwrap());
            }

            // call & send
            let mut results = (op.func)(args);
            for elem in results.drain(..).enumerate() {
                op.output[elem.0].send(elem.1).unwrap();
            }
        });
    }

    // the following is purely for testing
    // providing input to the DFG
    insertion_point.send(Box::from(Box::new(3))).unwrap();

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
