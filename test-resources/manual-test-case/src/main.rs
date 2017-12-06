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
use std::collections::HashMap;


fn main() {
    // let's just assume this function will be generated
    let runtime_data = ohuadata::generate();

    // TODO: Move the Arc generation here in order to be able to allocate enough space for the I/O channels when generating the operator struct

    // instantiate the operator vector with space for exactly n operators
    let mut operators: Vec<OhuaOperator> = Vec::with_capacity(runtime_data.graph.operators.len());

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

    let mut channels: Vec<(HashMap<u32, mpsc::Receiver<Box<GenericType>>>, HashMap<u32, Vec<mpsc::Sender<Box<GenericType>>>>)> = Vec::with_capacity(operators.len());

    for _ in 0..operators.len() {
        // are you fkin' serious
        channels.push((HashMap::new(), HashMap::new()));
    }

    // TODO: Make Arc recognition Enum based(?)
    for arc in runtime_data
            .graph
            .arcs
            .iter()
            .filter(|x| x.source.s_type == String::from("local")) {
        let (s, r) = mpsc::channel::<Box<GenericType>>();

        // place the receiver
        channels[(arc.target.operator - 1) as usize].0.insert(arc.target.index as u32, r);

        // place the sender
        if let types::ValueType::LocalVal(ref source) = arc.source.val {
            let sender_index: u32 = if source.index >= 0 {
                source.index as u32
            } else {
                0
            };

            if let Some(target) = channels[(source.operator - 1) as usize].1.get_mut(&sender_index) {
                // if there is at least 1 sender in place to transmit the value, just add the sender
                target.push(s);
                continue;
            }

            // if no sender is available for that slot (yet), add a new one
            channels[(source.operator - 1) as usize].1.insert(sender_index, vec![s]);
        } else {
            panic!("Encountered malformed ArcSource, type is `local` but type is EnvironmentVal.");
        }
    }

    println!("{:?}", channels);
    // TODO: now put these into the right operators, (maybe) move upper part to function

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
