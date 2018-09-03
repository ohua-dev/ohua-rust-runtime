use std::sync::mpsc;

use ohua_types::OhuaData;

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
        ValueType::EnvironmentVal(id) => id
        ValueType::LocalVal(id) => id.operator
    }
}

fn get_num_inputs(op:i32, arcs: Vec<Arc>) -> usize {
    arcs.iter().filter(|arc| arc.target.operator == op).count()
}

fn generate_in_arcs_vec() -> String {

}

// TODO extend to allow ops to have multiple outputs/outgoing arcs
fn code_generation(compiled::OhuaData) -> () {

    // generate the code for the function references
    let mut header = "use std::collections::LinkedList;\n\n".to_owned();



    let arc_template = |source target target_idx| = "let (sf_" + source + "_out, sf_" + target + "_in_" + target_idx + ") = mpsc::channel();\n"
    let sf_template = |in_arcs out_arc sfn| = "tasks.push_back(run_sf!(" + in_arcs + ", " + out_arc + ", " + sfn + "))";

    /**
        Generate the arc code. This yields:
        (let (sf_{op_id}_out, sf_{op_id}_in_{idx}) = mpsc::channel();)+
     */
    let mut arc_code = "".to_owned();
    for arc in compiled.graph.arcs.iter() {
        arc_code.push_str(&(arc_template(
            get_op_id(arc.source),
            arc.target.operator,
            arc.target.index)
        ));
    }

    /**
        Generate the sf code. This yields:
        let mut tasks: LinkedList<Task> = LinkedList::new();
        (tasks.append(run_sf!([{sf_{op_id}_in_{idx}}]*, sf_{op_id}_out, sfn));)+
     */
     let mut sf_code = "let mut tasks: LinkedList<Task> = LinkedList::new();\n".to_owned();
     for op in compiled.graph.operators.iter() {
         sf_code.push_str(&(sf_template(
             generate_in_arcs_vec(op.operatorId, compiled.graph.arcs), // this is not efficient but it works for now
             "sf_" + op.operatorId + "_out",
             op.operatorType.func
         )));
     }

     // the final call
     let run_it = "run_ohua(tasks)".to_owned();
     header.push_str(header)
           .push_str("\n\n")
           .push_str(arc_code)
           .push_str("\n")
           .push_str(sf_code)
           .push_str(run_it)
}

fn my_simple_sf(a: i32) -> i32 {
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

fn main() {
    value_test();
    code_gen_test();
}
