use std::sync::mpsc::{Receiver,Sender};

// FIXME the minimal implementation works on the Ord trait
pub fn smapFun<T>(in: Receiver<Vec<T>>, out: Sender<T>) -> () {
    unimplemented!();
}

// a fully explicit operator version
pub fn collect<T>(n: Receiver<i32>, data: Receiver<T>, out: Sender<Vec<T>>) -> () {
    loop {
        match n.recv() {
            Err(_) => {
                // channels are closed by Rust itself
            }
            Ok(num) => {
                let mut buffered = Vec::new();
                for _x in 0..num {
                    buffered.push(data.recv().unwrap());
                }
                out.send(buffered).unwrap();
            }
        }
    }
}

pub fn select<T>(decision: Receiver<bool>,
                 true_branch: Receiver<T>,
                 else_branch: Receiver<T>,
                 out: Sender<T>) -> () {
    unimplemented!();
}

// that's also a stateful function -> in fact this thing needs variadic arguments and therefore needs to be a macro
// FIXME this probably wants to become a procedural macro! (not that proc macros can also have the form 'scope!()')
macro_rules! scope {
    // FIXME this is not as trivial as it seems because we need different type parameters! and therefore need a recursive macro!
    ( $($input),+ ) => {
        pub fn <$(T),+>scope($($input),+) -> ($(T),+) {
            ($($input),+)
        }
    }
}

pub fn one_to_n<T>(n: Receiver<i32>, val: Receiver<T>, out: Sender<T>) -> () {
    unimplemented!();
}

// that's actually a stateful function
pub fn size<T>(&data:Vec<T>) -> i32 {
    data.len()
}

// stateful function
pub fn id<T>(data:T) -> T {
    data
}
