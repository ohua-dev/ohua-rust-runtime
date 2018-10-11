use std::sync::mpsc::Sender;
use std::thread;

// FIXME how do we get a result from this?
pub fn run_ohua(mut tasks: Vec<Box<Fn() -> () + Send + 'static>>) -> ()
// where
//     F: FnOnce() -> (),
//     F: Send + 'static,
{
    let mut handles = Vec::with_capacity(tasks.len());
    for task in tasks.drain(..) {
        handles.push(thread::spawn(move || {
            task();
        }));
    }

    for h in handles {
        if let Err(e) = h.join() {
            println!("Error: {:?}", e);
        }
    }
}
