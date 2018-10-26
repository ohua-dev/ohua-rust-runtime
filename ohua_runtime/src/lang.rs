use std::sync::mpsc::{Receiver, Sender};

// TODO the minimal implementation works on the Ord trait
#[allow(non_snake_case)]
pub fn smapFun<T>(inp: Receiver<Vec<T>>, out: Sender<T>) -> () {
    let vs = inp.recv().unwrap();
    for v in vs {
        out.send(v).unwrap();
    }
}

// a fully explicit operator version
pub fn collect<T>(n: &Receiver<i32>, data: &Receiver<T>, out: &Sender<Vec<T>>) -> () {
    match n.recv() {
        Err(_) => {
            // channels are closed by Rust itself
        }
        Ok(num) => {
            let mut buffered = Vec::new();
            for _x in 0..num {
                buffered.push(data.recv().unwrap());
            }
            out.send(buffered).unwrap();
        }
    }
}

pub fn select<T>(
    decision: Receiver<bool>,
    true_branch: Receiver<T>,
    else_branch: Receiver<T>,
    out: Sender<T>,
) -> () {
    let branch = if decision.recv().unwrap() {
        true_branch
    } else {
        else_branch
    };
    out.send(branch.recv().unwrap()).unwrap();
}

#[allow(non_snake_case)]
pub fn oneToN<T: Clone>(n: Receiver<i32>, val: Receiver<T>, out: Sender<T>) -> () {
    // TODO 2 more efficient implementations exist:
    //      1. send the key and the value once -> requires special input ports that are sensitive to that.
    //      2. send a batch -> requires input ports to understand the concept of a batch.
    // feels like the second option is the more general one (but also creates more data).
    // note: sharing the value is only possible of the function using it, does not mutate it! this can be yet another application for our knowledge base to find out which version to choose.
    let v = val.recv().unwrap();
    for _ in 0..(n.recv().unwrap()) {
        out.send(v.clone()).unwrap();
    }
}

// that's actually a stateful function
pub fn size<T>(data: Vec<T>) -> usize {
    data.len()
}

// stateful function
pub fn id<T>(data: T) -> T {
    data
}
