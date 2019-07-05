use std::marker::Send;
use std::sync::mpsc::{RecvError, SendError};
use std::thread;

pub mod arcs;
pub mod lang;

/// Error type representing possible errors when sending or receiving data via arcs.
#[derive(Debug)]
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
pub fn run_tasks(mut tasks: Vec<Box<dyn FnOnce() -> Result<(), RunError> + Send + 'static>>) -> () {
    let mut handles = Vec::with_capacity(tasks.len());
    for task in tasks.drain(..) {
        handles.push(thread::spawn(move || {
            let _ = task();
        }));
    }

    for h in handles {
        if let Err(_) = h.join() {
            eprintln!("[Error] A worker thread of an ohua algorithm has panicked!");
        }
    }
}
