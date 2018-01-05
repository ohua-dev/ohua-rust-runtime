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


fn sorted_recv_insertion(recvs: &mut Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, recv: mpsc::Receiver<Box<GenericType>>, target: u32) {
    let mut index: usize = 0;

    for &(ind, _) in recvs.iter() {
        if ind >= target {
            index = ind as usize;
            break;
        }
    }

    recvs.insert(index, (target, recv));
}

fn sorted_sender_insertion(senders: &mut Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>, sender: mpsc::Sender<Box<GenericType>>, target: u32) {
    let mut index: usize = 0;
    let mut exists = false;

    for &(ind, _) in senders.iter() {
        if ind == target {
            index = ind as usize;
            exists = true;
            break;
        } else if ind > target {
            index = ind as usize;
            break;
        }
    }

    if exists {
        // if there is at least 1 sender in place to transmit the value, just add the sender
        senders[index].1.push(sender);
    } else {
        // if no sender is available for that slot (yet), add a new one
        senders.insert(index, (target, vec![sender]));
    }
}


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

    // =========================== channel creation and placement ===========================

    // TODO: write a proper documentation for this data structure!
    let mut channels: Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)> = Vec::with_capacity(operators.len());

    for _ in 0..operators.len() {
        // are you fkin' serious
        channels.push((Vec::new(), Vec::new()));
    }

    // TODO: Make Arc recognition Enum based(?), move this into separate function
    for arc in runtime_data
            .graph
            .arcs
            .iter()
            .filter(|x| x.source.s_type == String::from("local")) {
        let (s, r) = mpsc::channel::<Box<GenericType>>();

        // place the receiver
        sorted_recv_insertion(&mut channels[(arc.target.operator - 1) as usize].0, r, arc.target.index as u32);

        // place the sender
        if let types::ValueType::LocalVal(ref source) = arc.source.val {
            // handle case when an operator only has one output arc (specified using "-1" as source)
            let sender_index: u32 = if source.index >= 0 {
                source.index as u32
            } else {
                0
            };

            sorted_sender_insertion(&mut channels[(source.operator - 1) as usize].1, s, sender_index);
        } else {
            panic!("Encountered malformed ArcSource, is defined as `local` but contains an EnvironmentVal.");
        }
    }

    // output port
    let (s, output_port) = mpsc::channel();
    channels[1].1.insert(0, (0, vec![s]));

    // TODO: move upper part to function
    for mut op_channels in channels.drain(..).enumerate() {
        operators[op_channels.0].input = (op_channels.1).0.drain(..).unzip::<u32, mpsc::Receiver<Box<GenericType>>, Vec<u32>, Vec<mpsc::Receiver<Box<GenericType>>>>().1;

        operators[op_channels.0].output = (op_channels.1).1.drain(..).unzip::<u32, Vec<mpsc::Sender<Box<GenericType>>>, Vec<u32>, Vec<Vec<mpsc::Sender<Box<GenericType>>>>>().1;
    }

    // ======================== end of channel creation and placement ========================

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
                // if op.output[elem.0].len() > 1 {
                //     cloning_send(elem.1, &op.output[elem.0]);
                // } else {
                    op.output[elem.0][0].send(elem.1).unwrap();
                // }
            }
        });
    }

    // ============ the following is purely for testing ============

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
