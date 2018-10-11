use std::sync::mpsc::Sender;
use std::thread;

// FIXME how do we get a result from this?
pub fn run_ohua(mut tasks: Vec<Box<Fn() -> () + Send + 'static>>) -> ()
// where
//     F: FnOnce() -> (),
//     F: Send + 'static,
{
    for task in tasks.drain(..) {
        thread::spawn(move || {
            task();
        });
    }
}

pub fn send<T: Copy>(val: T, outputs: Vec<&Sender<T>>) -> () {
    // option 1: we could borrow here and then it would fail if somebody tries to write to val. (pass-by-ref)
    // option 2: clone (pass-by-val)
    // this is something that our knowledge base could be useful for: check if any of the predecessor.
    // requires a mutable reference. if so then we need a clone for this predecessor.
    // borrowing across channels does not seem to work. how do I make a ref read-only in Rust? is this possible at all?
    match outputs.len() {
        0 => (), // drop
        1 => outputs[0].send(val).unwrap(),
        _ => for output in outputs {
            output.send(val.clone()).unwrap();
        },
    };
}
