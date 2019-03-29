use crate::arcs::ArcInput;
use crate::RunError;
use std::any::Any;
use std::iter::Iterator;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

#[allow(non_snake_case)]
pub fn smapFun<T: Any + 'static + Send, S: IntoIterator<Item = T> + 'static + Send>(
    inp: &Receiver<S>,
    data_out: &dyn ArcInput<T>,
    ctrl_out: &dyn ArcInput<(bool, isize)>,
    collect_out: &dyn ArcInput<usize>,
) -> Result<(), RunError> {
    let data = inp.recv()?.into_iter();
    let (_, size) = data.size_hint();
    match size {
        Some(s) => {
            // known size
            collect_out.dispatch(s)?;
            ctrl_out.dispatch((true, s as isize))?;
            for d in data {
                data_out.dispatch(d)?;
            }
        }
        None => {
            // unknown size -> generator-style
            let mut size = 0;
            for d in data {
                data_out.dispatch(d)?;
                ctrl_out.dispatch((false, 1))?;
                size = size + 1;
            }
            collect_out.dispatch(size)?;
            ctrl_out.dispatch((true, 0))?;
        }
    }
    Ok(())
}

pub fn collect<T: Send>(
    n: &Receiver<usize>,
    data: &Receiver<T>,
    out: &dyn ArcInput<Vec<T>>,
) -> Result<(), RunError> {
    let num = n.recv()?;
    let mut buffered = Vec::new();
    for _x in 0..num {
        buffered.push(data.recv()?);
    }
    out.dispatch(buffered)?;
    Ok(())
}

pub fn select<T: Send>(
    decision: &Receiver<bool>,
    true_branch: &Receiver<T>,
    else_branch: &Receiver<T>,
    out: &dyn ArcInput<T>,
) -> Result<(), RunError> {
    let branch = if decision.recv()? {
        true_branch
    } else {
        else_branch
    };
    out.dispatch(branch.recv()?)?;
    Ok(())
}

// this does not need destructuring operators.
#[allow(non_snake_case)]
pub fn ifFun(
    cond: &Receiver<bool>,
    ctrl_true: &dyn ArcInput<(bool, isize)>,
    ctrl_false: &dyn ArcInput<(bool, isize)>,
) -> Result<(), RunError> {
    if cond.recv()? {
        ctrl_true.dispatch((true, 1))?;
        ctrl_false.dispatch((true, 0))?;
    } else {
        ctrl_true.dispatch((true, 0))?;
        ctrl_false.dispatch((true, 1))?;
    }
    Ok(())
}

pub fn id<T>(data: T) -> T {
    data
}

#[allow(non_snake_case)]
pub fn seqFun<T: Send>(_: T) -> (bool, isize) {
    (true, 1)
}

#[allow(non_snake_case, unused_variables)]
pub fn recurFun<T: Send, S: Send>(
    call_actuals_in: &Receiver<T>,
    recur_cond_in: &Receiver<T>,
    recur_actuals_in: &Receiver<T>,
    recur_result_in: &Receiver<S>,
    ctrl_out: &dyn ArcInput<(bool, isize)>,
    recur_formals_out: &dyn ArcInput<T>,
    result_out: &dyn ArcInput<S>,
) -> Result<(), RunError> {
    // TODO this will have to be in the code generation as well!

    Ok(())
}

// a function to pass literals to operators
pub fn send_once<T>(t: T) -> Receiver<T> {
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
pub fn unitFn<A, F>(f: F, _signal: Unit) -> A
where
    F: Fn() -> A,
{
    f()
}
