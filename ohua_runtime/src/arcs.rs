//! Structures and methods for moving and exchanging data between operators.
use std::marker::Send;
use std::sync::mpsc::{SendError, Sender};

/// An arc that does not have a receiving side. Any data sent into this arc is dropped.
#[derive(Default)]
pub struct DeadEndArc {}

/// Central abstraction layer for all arcs. Every type of arc needs to implement this
/// as it is used to move data between operators.
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

/// A cloning dispatch queue as abstraction for output ports that serve more than one arc.
pub struct DispatchQueue<T: Send> {
    senders: Vec<Sender<T>>,
}

impl<T: Send> DispatchQueue<T> {
    pub fn new(senders: Vec<Sender<T>>) -> DispatchQueue<T> {
        DispatchQueue { senders: senders }
    }
}

impl<T: Send + Clone> ArcInput<T> for DispatchQueue<T> {
    fn dispatch(&self, t: T) -> Result<(), SendError<T>> {
        for sx in &self.senders {
            sx.send(t.clone())?;
        }

        Ok(())
    }
}
