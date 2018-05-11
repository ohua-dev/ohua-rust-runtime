#![allow(unused_variables)]

mod types;
mod generictype;
mod wrappers;
mod runtime;

use self::generictype::*;
use self::types::{Arc, ArcIdentifier, OhuaOperator, ValueType};

use std::thread::{self, Builder};
use std::sync::mpsc;

{ty_imports}


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
    let mut index: usize = senders.len();
    let mut exists = false;

    // iterate over the enumerated sender list and place the sender at the correct index
    for (vec_index, sender) in senders.iter().enumerate() {
        let &(ind, _) = sender;

        if ind == target {
            index = vec_index as usize;
            exists = true;
            break;
        } else if ind > target {
            index = vec_index as usize;
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
    /* This data structure is used to assign all receivers and senders to the correct operators
       before the actual runtime is started. Each operator has a pair of senders and receivers,
       bundled together. After initialization, this structure is consumed and the channels are
       distributed to the operators
    */
    let mut channels: Vec<(Vec<(u32, mpsc::Receiver<Box<GenericType>>)>, Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)>)> = Vec::with_capacity(op_count);
    let mut input_chans = Vec::with_capacity(input_targets.len());

    // initialize the channel matrix for all operators
    for _ in 0..op_count {
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

    // TWEAK: [Optimization] Move the Arc generation here in order to be able to
    // allocate enough space for the I/O channels when generating the operator struct

    // instantiate the operator vector with space for exactly n operators
    let mut operators: Vec<OhuaOperator> = Vec::with_capacity(runtime_data.graph.operators.len());

    // statically fill the operator struct
    for op in runtime_data.graph.operators {
        operators.push(OhuaOperator {
                           input: vec![],
                           output: vec![],
                           name: op.operatorType.qualified_name(),
                           func: op.operatorType.func,
                       })
    }

    // create and place channels for the arcs specified
    let (input_ports, mut channels, output_port) = generate_channels(operators.len(), &runtime_data.graph.arcs, &runtime_data.graph.return_arc, &runtime_data.graph.input_targets);

    for (index, mut op_channels) in channels.drain(..).enumerate() {
        operators[index].input = op_channels.0.drain(..).unzip::<u32, mpsc::Receiver<Box<GenericType>>, Vec<u32>, Vec<mpsc::Receiver<Box<GenericType>>>>().1;

        operators[index].output = op_channels.1;
    }

    // thread spawning
    for op in operators.drain(..) {
        Builder::new()
                .name(op.name.as_str().into())
                .spawn(move || 'threadloop: loop {
            let mut exiting = false;

            // receive the arguments from all senders
            let mut args = vec![];
            for (index, recv) in (&op.input).iter().enumerate() {
                if let Ok(content) = recv.recv() {
                    if !exiting {
                        args.push(content);
                    } else {
                        #[cold]
                        // when we are in `exiting` state, we should not be here...
                        eprintln!("[Error] Thread {} entered an inconsistent state. Some input Arcs are empty, others not.", thread::current().name().unwrap());
                        break 'threadloop;
                    }
                } else {
                    // when there are no messages left to receive, this operator is done
                    if !exiting {
                        // before entering the `exiting` state, make sure that this is valid behavior
                        if index > 0 {
                            #[cold]
                            eprintln!("[Error] Thread {} entered an inconsistent state. Some input Arcs are empty, others not.", thread::current().name().unwrap());
                            break 'threadloop;
                        } else {
                            exiting = true;
                        }
                    }
                }
            }

            // when we are in `exiting` state, kill gracefully
            if exiting {
                break 'threadloop;
            }

            // call function & send results
            let mut results = (op.func)(args);
            for &(ref port, ref senders) in &op.output {
                for sender in senders {
                    let element_to_send = results[*port as usize].pop().expect(&format!("Could not satisfy output port {} at {}", port, op.name));
                    sender.send(element_to_send).unwrap();
                }
            }
        }).unwrap();
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
