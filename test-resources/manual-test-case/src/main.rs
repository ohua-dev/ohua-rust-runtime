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

    // =========================== channel creation and placement ===========================

    // main argument arcs umformen
    // pro local arc channel einpflegen (mittels iter-search im Vec)
    // datenstruktur umkrempeln und umwandeln (in Runtime-Structure einbauen)
    // Datenstruktur umbauen


    let mut channels: Vec<(HashMap<u32, mpsc::Receiver<Box<GenericType>>>, HashMap<u32, Vec<mpsc::Sender<Box<GenericType>>>>)> = Vec::with_capacity(operators.len());

    for _ in 0..operators.len() w{
        // are you fkin' serious
        channels.push((HashMap::new(), HashMap::new()));
    }

    // TODO: Make Arc recognition Enum based(?), move this into separate function
    for arc in runtime_data
            .graph
            .arcs
            .iter()
            .filter(|x| x.source.s_type == String::from("local")) {
        // TODO: write a proper documentation for this data structure!
        let (s, r) = mpsc::channel::<Box<GenericType>>();

        // place the receiver
        channels[(arc.target.operator - 1) as usize].0.insert(arc.target.index as u32, r);

        // place the sender
        if let types::ValueType::LocalVal(ref source) = arc.source.val {
            // handle case when an operator only has one output arc (specified using "-1" as source)
            let sender_index: u32 = if source.index >= 0 {
                source.index as u32
            } else {
                0
            };

            // if there is at least 1 sender in place to transmit the value, just add the sender
            if let Some(target) = channels[(source.operator - 1) as usize].1.get_mut(&sender_index) {
                target.push(s);
                continue;
            }

            // if no sender is available for that slot (yet), add a new one
            channels[(source.operator - 1) as usize].1.insert(sender_index, vec![s]);
        } else {
            panic!("Encountered malformed ArcSource, is defined as `local` but contains an EnvironmentVal.");
        }
    }

    // env sources...
    let (insertion_point, r) = mpsc::channel();
    channels[0].0.insert(0, r);

    // output port
    let (s, output_port) = mpsc::channel();
    channels[1].1.insert(0, vec![s]);

    // TODO: move upper part to function
    for mut op_channels in channels.drain(..).enumerate() {
        operators[op_channels.0].input = {
            // extract the receivers, sort them and put them into the `input` vec
            let mut receivers: Vec<(u32, mpsc::Receiver<Box<GenericType>>)> = (op_channels.1).0.drain().collect();
            receivers.sort_by(|a, b| a.0.cmp(&b.0));
            let inputs = receivers.drain(..).map(|x| x.1).collect::<Vec<mpsc::Receiver<Box<GenericType>>>>();
            inputs
        };

        operators[op_channels.0].output = {
            // extract the senders, sort them and put them into the `output` vec
            let mut senders: Vec<(u32, Vec<mpsc::Sender<Box<GenericType>>>)> = (op_channels.1).1.drain().collect();
            senders.sort_by(|a, b| a.0.cmp(&b.0));
            let outputs = senders.drain(..).map(|x| x.1).collect::<Vec<Vec<mpsc::Sender<Box<GenericType>>>>>();
            outputs
        };
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
                if op.output[elem.0].len() > 1 {
                    cloning_send(elem.1, &op.output[elem.0]);
                } else {
                    op.output[elem.0][0].send(elem.1).unwrap();
                }
            }
        });
    }

    // ============ the following is purely for testing ============
    // providing input to the DFG
    insertion_point.send(Box::from(Box::new(3))).unwrap();

    // running...

    // finished! Gather output
    let res = output_port.recv().unwrap();

    println!("{:?}", Box::<i32>::from(res));
}
