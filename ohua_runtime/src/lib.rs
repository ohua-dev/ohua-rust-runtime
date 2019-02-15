#![feature(fnbox)]
use std::marker::Send;
use std::sync::mpsc::{RecvError, SendError};
use std::thread;

use std::boxed::FnBox;

pub mod lang;
pub mod arcs;

/// Error type representing possible errors when sending or receiving data via arcs.
pub enum RunError {
    SendFailed,
    RecvFailed,
}

impl<T: Send> From<SendError<T>> for RunError {
    fn from(_error: SendError<T>) -> Self {
        RunError::SendFailed
    }
}

impl From<RecvError> for RunError {
    fn from(_error: RecvError) -> Self {
        RunError::RecvFailed
    }
}

/// Central function to execute an algorithm.
///
/// The algorithm is provided as a set of tasks, each of which is going to be executed in a separate thread.
pub fn run_tasks(mut tasks: Vec<Box<FnBox() -> Result<(), RunError> + Send + 'static>>) -> () {
    let mut handles = Vec::with_capacity(tasks.len());
    for task in tasks.drain(..) {
        handles.push(thread::spawn(move || {
            let _ = task();
        }));
    }

    for h in handles {
        if let Err(_) = h.join() {
            println!("[Error] A worker thread of an ohua algorithm has panicked!");
        }
    }
}
