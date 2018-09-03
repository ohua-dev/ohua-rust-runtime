use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::ops::Add;

use ohua_types::OhuaData;
use ohua_types::ValueType;
use ohua_types::ArcSource;
use ohua_types::Arc;

/**
 Example operator: collect
 */

// a fully explicit operator version
fn collect<T>(n:Receiver<i32>, data:Receiver<T>, out:Sender<Vec<T>>) -> () {
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

pub trait Task {
    fn run(&self) -> ();
}

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

fn get_op_id(source:ArcSource) -> i32 {
    match source.val {
        ValueType::EnvironmentVal(i) => { i }
        ValueType::LocalVal(i) => { i.operator }
    }
}

fn get_num_inputs(op:i32, arcs:Vec<Arc>) -> usize {
    arcs.iter().filter(|arc| arc.target.operator == op).count()
}

fn generate_in_arcs_vec(op:i32, arcs:Vec<Arc>) -> String {
    let mut r = "[".to_owned();
    let n = get_num_inputs(op, arcs);
    for i in 0..(n-1) {
        r.push_str(&(i.to_string() + ", "));
    }
    if n > 0 {
        r.push_str(&(n-1).to_string());
    } else {
        // do nothing
    }
    r.push_str("]");
    r
}

// TODO extend to allow ops to have multiple outputs/outgoing arcs
fn code_generation(compiled:OhuaData) -> String {

    // generate the code for the function references
    let mut header = "".to_owned();

    // templates for arcs and stateful functions
    let arc_template = |source, target, target_idx| { "let (sf_{source}_out, sf_{target}_in_{target_idx}) = mpsc::channel();\n".replace("{source}",source)
                                                                                                                               .replace("{target}", target)
                                                                                                                               .replace("{target_idx}", target_idx)};
    let sf_template = |in_arcs, out_arc, sfn| { "tasks.push(run_sf!({in_arcs}, {out_arc}, {sfn}))".replace("{in_arcs}", in_arcs)
                                                                                                  .replace("{out_arc}", out_arc)
                                                                                                  .replace("{sfn}", sfn)};

    /**
        Generate the arc code. This yields:
        (let (sf_{op_id}_out, sf_{op_id}_in_{idx}) = mpsc::channel();)+
     */
    let mut arc_code = "".to_owned();
    for arc in compiled.graph.arcs.iter() {
        arc_code.push_str(
            &(arc_template(
                &get_op_id(arc.source).to_string(),
                &arc.target.operator.to_string(),
                &arc.target.index.to_string()
            ))
        );
    }

    /**
        Generate the sf code. This yields:
        let mut tasks: LinkedList<Task> = LinkedList::new();
        (tasks.append(run_sf!([{sf_{op_id}_in_{idx}}]*, sf_{op_id}_out, sfn));)+
     */
     let mut sf_code = "let mut tasks = Vec::new();\n".to_owned();
     for op in compiled.graph.operators.iter() {
         sf_code.push_str(
             &(sf_template(
                 &generate_in_arcs_vec(op.operatorId, compiled.graph.arcs), // this is not efficient but it works for now
                 &"sf_{op_id}_out".replace("{op_id}", &op.operatorId.to_string()),
                 &op.operatorType.func
             ))
         );
     }

     // the final call
     let run_it = "run_ohua(tasks)".to_owned();
     let mut code = "".to_owned();
     code.push_str(&header);
     code.push_str("\n\n");
     code.push_str(&arc_code);
     code.push_str("\n");
     code.push_str(&sf_code);
     code.push_str(&run_it);
     code
}

/**
 Test code starts here:
 */

fn my_simple_sf(a:i32) -> i32 {
    a + 5
}

fn value_test() {
    let (sender1, receiver1) = mpsc::channel();
    let (sender2, receiver2) = mpsc::channel();

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

pub fn run_typedgen_tests() {
    value_test();
    code_gen_test();
}
