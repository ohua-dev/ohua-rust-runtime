use super::types::OhuaOperator;
use std::thread;

pub fn sfn(op: OhuaOperator) {
    'threadloop: loop {
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
            } else if !exiting {
                // when there are no messages left to receive, this operator is done
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

        // when we are in `exiting` state, kill gracefully
        if exiting {
            break 'threadloop;
        }

        // call function & send results
        let mut results = (op.func)(args);
        for &(ref port, ref senders) in &op.output {
            for sender in senders {
                let element_to_send = results[*port as usize].pop().unwrap_or_else(|| {
                    panic!("Could not satisfy output port {} at {}", port, op.name)
                });
                sender.send(element_to_send).unwrap();
            }
        }
    }
}
