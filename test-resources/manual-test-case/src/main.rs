#![allow(unused_variables)]

mod types;
mod runtime;
mod wrappers;
mod ohuadata;

// has to be here, references the dev project (maybe refactor to use it as extern_crate?)
mod hello;

use runtime::*;

use std::thread;
use std::sync::mpsc;



fn main() {
    // let's just assume this function will be generated
    let runtime_data = ohuadata::generate();

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

    // channel creation
    // TODO: rework
    let (insertion_point, r1) = mpsc::channel();
    operators[0].input.push(r1);
    let (s2, r2) = mpsc::channel();
    operators[0].output.push(s2);
    operators[1].input.push(r2);
    let (s3, output_port) = mpsc::channel();
    operators[1].output.push(s3);

    for arc in runtime_data
            .graph
            .arcs
            .iter()
            .filter(|x| x.source.s_type == String::from("env")) {
        let (s, r) = mpsc::channel::<Box<GenericType>>();
        // place channel correctly
    }

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

    // ------------ the following is purely for testing ------------
    // providing input to the DFG
    insertion_point.send(Box::from(Box::new(3))).unwrap();

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
