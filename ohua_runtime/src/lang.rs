use std::sync::mpsc::{Receiver, Sender};
use super::RunError;
use std::iter::Iterator;
use std::any::Any;

#[allow(non_snake_case)]
pub fn smapFun<T: Any + 'static + Send, S: Iterator<Item = T> + 'static + Send>
    (inp: &Receiver<S>, data_out: &Sender<T>, ctrl_out: &Sender<(bool, usize)>, collect_out: &Sender<usize>)
    -> Result<(), RunError> {
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

pub struct CtrlState<T>{
    vars: Vec<T>,
    renew: bool,
}

pub fn ctrl_state<T: Send + Clone>() -> CtrlState<T> {
    CtrlState{
        vars: vec![],
        renew: true,
    }
}

// TODO heterogeneous lists in Rust don't come easy.
//      we really have to put this operator into our code generation.
//      this might work for now: https://github.com/lloydmeta/frunk
pub fn ctrl<T: Send + Clone>(state: &mut CtrlState<T>, ctrl_inp: &Receiver<(bool,isize)>,
                     vars_inp: &Vec<Receiver<T>>, outs: &Vec<Sender<T>>) -> Result<(), RunError> {

    let (renew_next_time, count) = ctrl_inp.recv()?;
    if !state.renew {
        // TODO Turn unwraps into the proper runtime errors
        let vars:Vec<T> = vars_inp.iter().map(|var_inp| var_inp.recv().unwrap()).collect();
        state.vars = vars;
    } else {
        // renuse the captured vars
    };

    // update the state
    state.renew = renew_next_time;

    for _ in 0..count {
        state.vars.iter()
                  .zip(outs)
                  .for_each(|(var, out)| out.send(var.clone()).unwrap());
    }
    Ok(())
}

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

#[allow(non_snake_case)]
pub fn seqFun<T: Send>(_: T) -> (bool,isize) {
    (true, 1)
}
