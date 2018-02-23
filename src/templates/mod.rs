#![allow(unused_variables)]

mod types;
mod generictype;
mod wrappers;
mod runtime;

use self::generictype::*;
use self::types::{Arc, ArcIdentifier, OhuaOperator, ValueType};

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


fn generate_channels(op_count: usize, arcs: &Vec<Arc>, return_arc: &ArcIdentifier, input_targets: &Vec<ArcIdentifier>) -> (Vec<mpsc::Sender<Box<GenericType>>>, Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)>, mpsc::Receiver<Box<GenericType>>) {
    // TODO: write a proper documentation for this data structure!
    let mut channels: Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)> = Vec::with_capacity(op_count);
    let mut input_chans = Vec::with_capacity(input_targets.len());

    for _ in 0..op_count {
        // are you fkin' serious
        channels.push((Vec::new(), Vec::new()));
    }

    for arc in arcs
            .iter()
            .filter(|x| x.source.s_type == String::from("local")) {
        let (s, r) = mpsc::channel::<Box<GenericType>>();

        // place the receiver
        sorted_recv_insertion(&mut channels[(arc.target.operator - 1) as usize].0, r, arc.target.index as u32);

        // place the sender
        if let ValueType::LocalVal(ref source) = arc.source.val {
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

    // place the input ports, if any
    for target in input_targets {
        let (sx, rx) = mpsc::channel();

        sorted_recv_insertion(&mut channels[(target.operator - 1) as usize].0, rx, target.index as u32);
        input_chans.push(sx);
    }

    // place the output port
    let (s, output_port) = mpsc::channel();
    let sender_index: u32 = if return_arc.index >= 0 {
        return_arc.index as u32
    } else {
        0
    };

    sorted_sender_insertion(&mut channels[(return_arc.operator - 1) as usize].1, s, sender_index);

    (input_chans, channels, output_port)
}


pub fn ohua_main({input_args}) -> {return_type} {
    let runtime_data = runtime::generate();

    // TODO: [Optimization] Move the Arc generation here in order to be able to
    // allocate enough space for the I/O channels when generating the operator struct

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
    let (input_ports, mut channels, output_port) = generate_channels(operators.len(), &runtime_data.graph.arcs, &runtime_data.graph.return_arc, &runtime_data.graph.input_targets);

    for mut op_channels in channels.drain(..).enumerate() {
        operators[op_channels.0].input = (op_channels.1).0.drain(..).unzip::<u32, mpsc::Receiver<Box<GenericType>>, Vec<u32>, Vec<mpsc::Receiver<Box<GenericType>>>>().1;

        operators[op_channels.0].output = (op_channels.1).1.drain(..).unzip::<u32, Vec<mpsc::Sender<Box<GenericType>>>, Vec<u32>, Vec<Vec<mpsc::Sender<Box<GenericType>>>>>().1;
    }

    // thread spawning
    for op in operators.drain(..) {
        thread::spawn(move || 'threadloop: loop {
            // receive arguments
            let mut args = vec![];
            for recv in &op.input {
                if let Ok(content) = recv.recv() {
                    args.push(content);
                } else {
                    // TODO: Implement check whether *all* channels are empty
                    // when there are no messages left to receive, we are done
                    break 'threadloop;
                }
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

    // provide the operators with input from the function arguments, if any
    {send_input}
    // after sending all input data, drop the senders to start the graceful
    // dissolution of the data flow network
    drop(input_ports);

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    // return the result
    *Box::<{return_type}>::from(res)
}
