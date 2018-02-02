#![allow(unused_variables)]

mod types;
mod generictype;
mod wrappers;
mod runtime;

use self::generictype::*;
use self::types::OhuaOperator;

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


fn generate_channels(op_count: usize, arcs: &Vec<types::Arc>) -> (Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)>, mpsc::Receiver<Box<GenericType>>) {
    // TODO: write a proper documentation for this data structure!
    let mut channels: Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)> = Vec::with_capacity(op_count);

    for _ in 0..op_count {
        // are you fkin' serious
        channels.push((Vec::new(), Vec::new()));
    }

    // TODO: Make Arc recognition Enum based(?), move this into separate function
    for arc in arcs
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

    (channels, output_port)
}


pub fn ohua_main() {
    // let's just assume this function will be generated
    let runtime_data = runtime::generate();

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

    // create and place channels for the arcs specified
    let (mut channels, output_port) = generate_channels(operators.len(), &runtime_data.graph.arcs);

    for mut op_channels in channels.drain(..).enumerate() {
        operators[op_channels.0].input = (op_channels.1).0.drain(..).unzip::<u32, mpsc::Receiver<Box<GenericType>>, Vec<u32>, Vec<mpsc::Receiver<Box<GenericType>>>>().1;

        operators[op_channels.0].output = (op_channels.1).1.drain(..).unzip::<u32, Vec<mpsc::Sender<Box<GenericType>>>, Vec<u32>, Vec<Vec<mpsc::Sender<Box<GenericType>>>>>().1;
    }

    // thread spawning
    for op in operators.drain(..) {
        thread::spawn(move || {
            // receive arguments
            let mut args = vec![];
            for recv in op.input {
                args.push(recv.recv().unwrap());
            }

            // call function & send results
            let mut results = (op.func)(args);
            for (index, mut element_vec) in results.drain(..).enumerate() {
                for (arc, msg) in element_vec.drain(..).enumerate() {
                    // TODO: What was this check good for? Can be removed?
                    if op.output.len() > 0 {
                        op.output[index][arc].send(msg).unwrap();
                    }
                }
            }
        });
    }

    // ============ the following is purely for testing ============

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
