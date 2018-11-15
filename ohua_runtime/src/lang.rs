use std::sync::mpsc::{Receiver, Sender};
use super::RunError;

// TODO the minimal implementation works on the Ord trait
#[allow(non_snake_case)]
pub fn smapFun<T: Send>(inp: &Receiver<Vec<T>>, out: &Sender<T>) -> Result<(), RunError> {
    let vs = inp.recv()?;
    for v in vs {
        out.send(v)?;
    }
    Ok(())
}

// a fully explicit operator version
pub fn collect<T: Send>(n: &Receiver<usize>, data: &Receiver<T>, out: &Sender<Vec<T>>) -> Result<(), RunError> {
    let num = n.recv()?;
    let mut buffered = Vec::new();
    for _x in 0..num {
        buffered.push(data.recv()?);
    }
    out.send(buffered)?;
    Ok(())
}

pub fn select<T: Send>(
    decision: &Receiver<bool>,
    true_branch: &Receiver<T>,
    else_branch: &Receiver<T>,
    out: &Sender<T>,
) -> Result<(), RunError> {
    let branch = if decision.recv()? {
        true_branch
    } else {
        else_branch
    };
    out.send(branch.recv()?)?;
    Ok(())
}

#[allow(non_snake_case)]
pub fn oneToN<T: Send + Clone>(n: &Receiver<usize>, val: &Receiver<T>, out: &Sender<T>) -> Result<(), RunError> {
    // TODO 2 more efficient implementations exist:
    //      1. send the key and the value once -> requires special input ports that are sensitive to that.
    //      2. send a batch -> requires input ports to understand the concept of a batch.
    // feels like the second option is the more general one (but also creates more data).
    // note: sharing the value is only possible of the function using it, does not mutate it! this can be yet another application for our knowledge base to find out which version to choose.
    let v = val.recv()?;
    for _ in 0..(n.recv()?) {
        out.send(v.clone())?;
    }
    Ok(())
}

// that's actually a stateful function
pub fn size<T>(data: Vec<T>) -> usize {
    data.len()
}

// stateful function
pub fn id<T>(data: T) -> T {
    data
}

// reference implementation: https://github.com/ohua-dev/ohua-jvm-runtime/blob/master/src/java/ohua/lang/IfSupport.java
// issue in core: https://github.com/ohua-dev/ohua-core/issues/21
pub fn bool(
    b: &Receiver<bool>,
    pos: &Sender<bool>,
    neg: &Sender<bool>,
    select_input: &Sender<bool>,
) -> Result<(), RunError> {
    let v = b.recv()?;
    pos.send(v)?;
    neg.send(!v)?;
    select_input.send(v)?;
    Ok(())
}

pub fn seq<T: Send>(_: T) -> bool {
    true
}
