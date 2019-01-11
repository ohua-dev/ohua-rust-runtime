use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use super::RunError;
use std::iter::Iterator;
use std::any::Any;

#[allow(non_snake_case)]
pub fn smapFun<T: Any + 'static + Send, S: Iterator<Item = T> + 'static + Send>
    (inp: &Receiver<S>,
     data_out: &Sender<T>,
     ctrl_out: &Sender<(bool, usize)>,
     collect_out: &Sender<usize>) -> Result<(), RunError> {
    let data = inp.recv()?;
    let (_, size) = data.size_hint();
    match size {
        Some(s) => {
            // known size
            collect_out.send(s)?;
            ctrl_out.send((true,s))?;
            for d in data { data_out.send(d)?; }
        }
        None => {
            // unknown size -> generator-style
            let mut size = 0;
            for d in data {
                data_out.send(d)?;
                ctrl_out.send((false,1))?;
                size = size +1;
            }
            collect_out.send(size)?;
            ctrl_out.send((true,0))?;
        }
    }
    Ok(())
}

pub fn collect<T: Send>(n: &Receiver<usize>,
                        data: &Receiver<T>,
                        out: &Sender<Vec<T>>) -> Result<(), RunError> {
    let num = n.recv()?;
    let mut buffered = Vec::new();
    for _x in 0..num {
        buffered.push(data.recv()?);
    }
    out.send(buffered)?;
    Ok(())
}

pub fn select<T: Send>(decision: &Receiver<bool>,
                       true_branch: &Receiver<T>,
                       else_branch: &Receiver<T>,
                       out: &Sender<T>) -> Result<(), RunError> {
    let branch = if decision.recv()? {
        true_branch
    } else {
        else_branch
    };
    out.send(branch.recv()?)?;
    Ok(())
}

// this does not need destructuring operators.
#[allow(non_snake_case)]
pub fn ifFun(cond: &Receiver<bool>,
             ctrl_true: &Sender<(bool,isize)>,
             ctrl_false: &Sender<(bool,isize)>) -> Result<(), RunError> {
    if cond.recv()? {
        ctrl_true.send((true, 1))?;
        ctrl_false.send((true, 0))?;
    } else {
        ctrl_true.send((true, 0))?;
        ctrl_false.send((true, 1))?;
    }
    Ok(())
}

// this is more concise but requires nth operators for destructuring.
// #[allow(non_snake_case)]
// pub fn ifFun(cond: bool) -> ((bool,isize), (bool,isize)) {
//     if cond {
//         ((true, 1),(true, 0))
//     } else {
//         ((true, 0),(true, 1))
//     }
// }

pub fn id<T>(data: T) -> T {
    data
}

#[allow(non_snake_case)]
pub fn seqFun<T: Send>(_: T) -> (bool,isize) {
    (true, 1)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
pub fn recurFun<T: Send, S: Send>(call_actuals_in: &Receiver<T>,
                                  recur_cond_in: &Receiver<T>,
                                  recur_actuals_in: &Receiver<T>,
                                  recur_result_in: &Receiver<S>,
                                  ctrl_out: &Sender<(bool,isize)>,
                                  recur_formals_out: &Sender<T>,
                                  result_out: &Sender<S>) -> Result<(), RunError> {
    // TODO this will have to be in the code generation as well!

    Ok(())
}

// a function to pass literals to operators
pub fn send_once<T>(t:T) -> Receiver<T> {
    let (snd, rcv) = channel();
    snd.send(t).unwrap();
    rcv
}

/**
 * Translating Unit, along with functions that implicitly take a Unit argument,
 * i.e., functions that take no arguments at all.
 */

// the type
#[derive(Debug, Clone)]
pub struct Unit {}

// a function that wraps around functions that receive a ().
#[allow(non_snake_case)]
pub fn unitFn<A,F>(f:F, _signal:Unit) -> A
where F: Fn() -> A {
    f()
}
