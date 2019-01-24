#![feature(fnbox)]
use std::marker::Send;
use std::sync::mpsc::{RecvError, SendError, Sender};
use std::thread;

use std::boxed::FnBox;

pub mod lang;

#[derive(Default)]
pub struct DeadEndArc {}

pub trait ArcInput<T> {
    fn dispatch(&self, t: T) -> Result<(), SendError<T>>;
}

impl<T> ArcInput<T> for Sender<T> {
    fn dispatch(&self, t: T) -> Result<(), SendError<T>> {
        self.send(t)
    }
}

impl<T: Send> ArcInput<T> for DeadEndArc {
    fn dispatch(&self, _t: T) -> Result<(), SendError<T>> {
        // drop
        Ok(())
    }
}

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
