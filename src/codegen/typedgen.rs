#![allow(unused_macros, dead_code, unused_doc_comments)]
use std::sync::mpsc::{Receiver, Sender};

use ohua_types::Arc;
use ohua_types::OhuaData;
use ohua_types::ValueType;

/**
 Example operator: collect
 */

// a fully explicit operator version
fn collect<T>(n: Receiver<i32>, data: Receiver<T>, out: Sender<Vec<T>>) -> () {
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

trait Task {
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
    ($e:expr) => {
        $e
    };
}

// TODO turn this into a loop and exit when an error on the input channel occurs
// TODO use a vector of outputs (create a normal output function because all the type related stuff happens until the sf was executed)
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

fn get_op_id(val: &ValueType) -> &i32 {
    match val {
        ValueType::EnvironmentVal(i) => i,
        ValueType::LocalVal(i) => &(i.operator),
    }
}

fn get_num_inputs(op: &i32, arcs: &Vec<Arc>) -> usize {
    arcs.iter()
        .filter(|arc| &(arc.target.operator) == op)
        .count()
}

fn generate_in_arcs_vec(op: &i32, arcs: &Vec<Arc>) -> String {
    let mut r = "[".to_owned();
    let n = get_num_inputs(&op, &arcs);
    if n > 0 {
        for i in 0..(n - 1) {
            r.push_str(&(format!("sf_{}_in_{},", op.to_string(), i.to_string())));
        }
        r.push_str(&format!(
            "sf_{}_in_{},",
            op.to_string(),
            (n - 1).to_string()
        ));
    } else {
        // do nothing
    }
    r.push_str("]");
    r
}

// TODO extend to allow ops to have multiple outputs/outgoing arcs
pub fn code_generation(compiled: OhuaData) -> String {
    // generate the code for the function references
    // TODO import statements
    let mut header = "".to_owned();

    // templates for arcs and stateful functions
    let arc_template = |source, target, target_idx| {
        format!(
            "let (sf_{}_out, sf_{}_in_{}) = mpsc::channel();\n",
            source, target, target_idx
        )
    };
    let sf_template = |in_arcs, out_arc, sfn| {
        format!("tasks.push(run_sf!({}, {}, {}));\n", in_arcs, out_arc, sfn)
    };

    /**
        Generate the arc code. This yields:
        (let (sf_{op_id}_out, sf_{op_id}_in_{idx}) = mpsc::channel();)+
     */
    let mut arc_code = "".to_owned();
    for arc in compiled.graph.arcs.iter() {
        arc_code.push_str(
            &(arc_template(
                get_op_id(&(arc.source.val)).to_string(),
                arc.target.operator.to_string(),
                arc.target.index.to_string(),
            )),
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
                generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs)), // this is not efficient but it works for now
                format!("sf_{}_out", op.operatorId.to_string()),
                &op.operatorType.func,
            )),
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;

    use ohua_types::Arc;
    use ohua_types::ArcIdentifier;
    use ohua_types::ArcSource;
    use ohua_types::DFGraph;
    use ohua_types::OhuaData;
    use ohua_types::OpType;
    use ohua_types::Operator;
    use ohua_types::OperatorType;
    use ohua_types::ValueType;

    fn my_simple_sf(a: i32) -> i32 {
        a + 5
    }

    #[test]
    fn sf_macro_value_test() {
        let (sender1, receiver1) = mpsc::channel();
        let (sender2, receiver2) = mpsc::channel();

        sender1.send(5).unwrap();
        run_sf!([receiver1], sender2, my_simple_sf);

        let result = receiver2.recv().unwrap();
        println!("Result: {}", result);
        assert!(result == 10);
    }

 #[cfg(test)]
 mod tests {
     use super::*;

     use std::sync::mpsc;

     use ohua_types::OhuaData;
     use ohua_types::ValueType;
     use ohua_types::ArcSource;
     use ohua_types::Operator;
     use ohua_types::Arc;
     use ohua_types::OperatorType;
     use ohua_types::OpType;
     use ohua_types::ArcIdentifier;
     use ohua_types::DFGraph;

     fn my_simple_sf(a:i32) -> i32 {
         a + 5
     }

     #[test]
     fn sf_macro_value_test() {
         let (sender1, receiver1) = mpsc::channel();
         let (sender2, receiver2) = mpsc::channel();

         sender1.send(5).unwrap();
         run_sf!([receiver1], sender2, my_simple_sf);

         let result = receiver2.recv().unwrap();
         println!("Result: {}", result);
         assert!(result == 10);
     }

     #[test]
     fn sf_macro_code_test() {
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

     #[test]
     fn full_code_gen_test() {
         let compiled = OhuaData {
             graph: DFGraph {
                 operators: vec![
                     Operator {
                         operatorId: 0,
                         operatorType: OperatorType {
                             qbNamespace: Vec::new(),
                             qbName: "none".to_string(),
                             func: "some_sfn".to_string(),
                             op_type: OpType::SfnWrapper,
                         },
                     },
                     Operator {
                         operatorId: 1,
                         operatorType: OperatorType {
                             qbNamespace: Vec::new(),
                             qbName: "none".to_string(),
                             func: "some_other_sfn".to_string(),
                             op_type: OpType::SfnWrapper,
                         },
                     },
                 ],
                 arcs: vec![Arc {
                     target: ArcIdentifier {
                         operator: 1,
                         index: 0,
                     },
                     source: ArcSource {
                         s_type: "".to_string(),
                         val: ValueType::LocalVal(ArcIdentifier {
                             operator: 0,
                             index: -1,
                         }),
                     },
                 }],
                 return_arc: ArcIdentifier {
                     operator: 1,
                     index: -1,
                 },
                 input_targets: Vec::new(),
             },
             mainArity: 1,
             sfDependencies: Vec::new(),
         };
         let generated = code_generation(compiled);
         println!("Generated code:\n\n{}\n\n", &generated);
     }
 }
