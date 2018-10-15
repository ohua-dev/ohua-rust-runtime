use std::thread;
use std::sync::mpsc::RecvError;

pub fn run_tasks(mut tasks: Vec<Box<Fn() -> Result<(), RecvError> + Send + 'static>>) -> ()
{
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
