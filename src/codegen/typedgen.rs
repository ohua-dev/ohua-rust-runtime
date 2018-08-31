use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

/*
The code below compiles and runs at https://play.rust-lang.org/?gist=6bfbf0b7cea3031437bdf94e11bcca61&version=stable&mode=debug&edition=2015
It shows a macro to create type safe code for stateful functions.
TODO:
  - This needs to be extended to have the "operator" of the stateful function loop until the input channels are closed.
  - The code in run_sf should actually create and return a task. A task is just an abstraction/trait for us to plug the ops onto threads and run them.
  - We need a similar macro for running built-in operators.
  
Check out the construction of the macro. It allows to write test cases for the code to be generated.
(In Clojure/Lisp there is macroexpand which is missing in Rust, so I used this little trick here.)
*/


macro_rules! id {
    ($e:expr) => { $e }
}

macro_rules! run_sf {
  ( [$($input:ident),*], $output:ident, $sf:ident) => { run_sf!([$($input),*], $output, $sf, id) };
  ( [$($input:ident),*],
    $output:ident,
    $sf:ident,
    $trace:ident ) => {
        $trace!({
            let r = $sf( $($input.recv().unwrap()),* );
            $output.send(r).unwrap()
            });
        }
}

fn my_simple_sf(a: i32) -> i32 {
    a + 5
}

fn value_test() {
    let (sender1, receiver1): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    let (sender2, receiver2): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    sender1.send(5).unwrap();
    run_sf!([receiver1], sender2, my_simple_sf);

    let result = receiver2.recv().unwrap();
    println!("Result: {}", result);
    assert!(result == 10);
}

fn code_gen_test() {
    //let (sender1, receiver1): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    //let (sender2, receiver2): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    println!(
        "{:?}",
        stringify!(run_sf!([receiver1], sender2, my_simple_sf))
    );
    let a = run_sf!([receiver1], sender2, my_simple_sf, stringify);
    println!("{:?}", a);
    assert!(a == "{\nlet r = my_simple_sf ( receiver1 . recv (  ) . unwrap (  ) ) ; sender2 . send\n( r ) . unwrap (  ) }")
}

fn main() {
    value_test();
    code_gen_test();
}
