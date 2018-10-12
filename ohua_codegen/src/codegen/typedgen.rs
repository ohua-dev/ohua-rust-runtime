#![allow(unused_doc_comments)]
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};

use ohua_types::ValueType::{EnvironmentVal, LocalVal};
use ohua_types::{Arc, ArcSource, OhuaData, OpType, Operator, OperatorType, ValueType};

use proc_macro2::{Ident, Span, TokenStream};

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

fn get_num_outputs(op: &i32, arcs: &Vec<Arc>) -> usize {
    arcs.iter()
        .filter(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => &(a_id.operator) == op,
        })
        .count()
}

fn get_outputs(op: &i32, arcs: &Vec<Arc>) -> Vec<i32> {
    let mut t: Vec<i32> = arcs
        .iter()
        .filter(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => &(a_id.operator) == op,
        })
        .map(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => a_id.index,
        })
        .collect();
    t.sort();
    t
}

fn get_out_arcs<'a>(op: &i32, arcs: &'a Vec<Arc>) -> Vec<&'a Arc> {
    let t = arcs
        .iter()
        .filter(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => &(a_id.operator) == op,
        })
        .collect();
    t
}

fn get_out_index_from_source(src: &ArcSource) -> &i32 {
    match src.val {
        EnvironmentVal(ref i) => i,
        LocalVal(ref arc_id) => &arc_id.index,
    }
}

fn generate_var_for_out_arc(op: &i32, idx: &i32, ops: &Vec<Operator>) -> String {
    let op_spec = ops
        .iter()
        .find(|o| &(o.operatorId) == op)
        // FIXME This may fail for environment variables.
        .expect(&format!(
            "Ohua compiler invariant broken: Operator not registered: {}",
            op
        ));
    let computed_idx = match &(op_spec.operatorType.op_type) {
        OpType::SfnWrapper => {
            assert!(idx == &-1);
            &0
        }
        OpType::OhuaOperator(_) => {
            assert!(idx >= &0);
            idx
        }
    };

    format!("sf_{}_out_{}", op.to_string(), computed_idx.to_string())
}

fn generate_out_arc_var(arc: &Arc, ops: &Vec<Operator>) -> Ident {
    let out_idx = get_out_index_from_source(&(arc.source));
    let src_op = get_op_id(&(arc.source.val));
    let out_port = generate_var_for_out_arc(src_op, out_idx, &ops);

    let in_port = generate_var_for_in_arc(&(arc.target.operator), &(arc.target.index));

    /**
    This enforces the following invariant in Ohua/dataflow:
    An input port can only have one incoming arc. So it is enough to use the op-id and the input-port-id
    to generate a unique variable name.
    However, it is possible for one output port to have more than one outgoing arc.
    As such, using the only the op-id in combination with the output-port-id will not work because
    there are multiple arcs with the same source and has the variable name would be generated
    multiple times.
    We simply make this name unique again by the fact that each arc is unique and an arc is uniquely
    identified by its source op/output-port and the target op/input-port.
     */
    Ident::new(
        &format!("{}__{}", out_port.to_string(), in_port.to_string()),
        Span::call_site(),
    )
}

fn generate_var_for_in_arc(op: &i32, idx: &i32) -> Ident {
    assert!(idx > &-2);
    let index = match idx {
        -1 => "ctrl".to_string(),
        _ => idx.to_string(),
    };
    Ident::new(
        &format!("sf_{}_in_{}", op.to_string(), index),
        Span::call_site(),
    )
}

fn generate_in_arcs_vec(op: &i32, arcs: &Vec<Arc>) -> Vec<Ident> {
    let n = get_num_inputs(&op, &arcs);
    // TODO handle control arcs. (control arc := target_idx = -1)
    (0..n)
        .map(|i| generate_var_for_in_arc(op, &(i as i32)))
        .collect()
}

fn generate_out_arcs_vec(op: &i32, arcs: &Vec<Arc>, ops: &Vec<Operator>) -> Vec<Ident> {
    get_out_arcs(&op, &arcs)
        .iter()
        .map(|arc| generate_out_arc_var(arc, ops))
        .collect()
}

pub fn generate_arcs(compiled: &OhuaData) -> TokenStream {
    let outs = compiled
        .graph
        .arcs
        .iter()
        .map(|arc| generate_out_arc_var(&arc, &(compiled.graph.operators)));
    let ins = compiled
        .graph
        .arcs
        .iter()
        .map(|arc| generate_var_for_in_arc(&(arc.target.operator), &(arc.target.index)));

    quote!{
        #(let (#outs, #ins) = std::sync::mpsc::channel();)*
    }
}

fn get_call_reference(op_type: &OperatorType) -> Ident {
    // according to the following reference, the name of the function is also an Ident;
    // https://docs.rs/syn/0.15/syn/struct.ExprMethodCall.html
    // but the Ident can not be ns1::f. so how are these calls parsed then?
    // if this happens then we might have to decompose qbName into its name and the namespace.
    Ident::new(&op_type.qbName, Span::call_site())
}

pub fn generate_ops(compiled: &OhuaData) -> TokenStream {
    let ops = compiled.graph.operators.iter().filter(|o| {
        (match o.operatorType.op_type {
            OpType::OhuaOperator(_) => true,
            _ => false,
        })
    });
    let op_codes: Vec<TokenStream> = ops
        .map(|op| {
            let mut in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
            let mut out_arcs = generate_out_arcs_vec(
                &(op.operatorId),
                &(compiled.graph.arcs),
                &(compiled.graph.operators),
            );
            let op_name = get_call_reference(&op.operatorType);

            in_arcs.append(&mut out_arcs);
            assert!(find_control_input(&(op.operatorId), &compiled.graph.arcs).is_none());

            if in_arcs.len() > 0 {
                quote!{
                    loop{
                        // FIXME do we really have a valid design to handle control input for operators?
                        //       how do we drain the inputs if the op dequeues n packets from some input channel???
                        // if #ctrl.recv().unboxed() {
                        #op_name(#(&#in_arcs),*);
                        // } else {
                        //     // skip
                        // }
                    }
                }
            } else {
                quote!{ #op_name() }
            }
        })
        .collect();

    quote!{
        #(tasks.push(|| { #op_codes }); )*
    }
}

fn find_control_input(op: &i32, arcs: &Vec<Arc>) -> Option<Ident> {
    arcs.into_iter()
        .find(|arc| &(arc.target.operator) == op && arc.target.index == -1)
        .map(|ctrl_arc| {
            generate_var_for_in_arc(&(ctrl_arc.target.operator), &(ctrl_arc.target.index))
        })
}

pub fn generate_sfns(compiled: &OhuaData) -> TokenStream {
    let sfns = compiled
        .graph
        .operators
        .iter()
        .filter(|&o| match o.operatorType.op_type {
            OpType::SfnWrapper => true,
            _ => false,
        });
    let sf_codes: Vec<TokenStream> = sfns
        .map(|op| {
            let in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
            let out_arcs = generate_out_arcs_vec(
                &op.operatorId,
                &(compiled.graph.arcs),
                &(compiled.graph.operators),
            );
            let sf = get_call_reference(&op.operatorType);
            let arcs = in_arcs.clone(); // can't reuse var in quote!
            let r = Ident::new(&"r", Span::call_site());
            let send = generate_send(&r, &out_arcs);

            let ctrl_port = find_control_input(&(op.operatorId), &compiled.graph.arcs);

            let drain_arcs = in_arcs.clone();
            let num_in_arcs = in_arcs.len();
            let drain_inputs = quote!{ #(#drain_arcs.recv()?;)* };
            let call_code = quote!{ #sf( #(#in_arcs.recv()?),* ) };
            let sfn_code = quote!{ let #r = #call_code; #send };

            if num_in_arcs > 0 {
                match &ctrl_port {
                    None => quote!{ loop { #sfn_code } },
                    Some(p) => quote!{ loop { if #p.recv()? { #sfn_code} else { #drain_inputs } } },
                }
            } else {
                match &ctrl_port {
                    None => quote!{ #sfn_code Ok(()) },
                    Some(p) => {
                        quote!{ loop { if #p.recv()? { #sfn_code } else { /* Drop call */ } } }
                    }
                }
            }
        })
        .collect();

    quote!{
        let mut tasks: Vec<Box<Fn() -> Result<(), RecvError> + Send + 'static>> = Vec::new();
        #(tasks.push(Box::new(move || { #sf_codes })); )*
    }
}

fn generate_send(r: &Ident, outputs: &Vec<Ident>) -> TokenStream {
    // option 1: we could borrow here and then it would fail if somebody tries to write to val. (pass-by-ref)
    // option 2: clone (pass-by-val)
    // this is something that our knowledge base could be useful for: check if any of the predecessor.
    // requires a mutable reference. if so then we need a clone for this predecessor.
    // borrowing across channels does not seem to work. how do I make a ref read-only in Rust? is this possible at all?
    match outputs.len() {
        0 => quote!{}, // drop
        1 => {
            let o = &outputs[0];
            quote!{ #o.send(#r).unwrap(); }
        }
        _ => {
            let results: Vec<Ident> = outputs.iter().map(|x| r.clone()).collect();
            quote!{
                #(#outputs.send(#results.clone()).unwrap());*;
                // for output in [#(#outputs),*].iter() {
                //     output.send(#r.clone()).unwrap();
                // }
            }
        }
    }
}

fn generate_app_namespaces(operators: &Vec<Operator>) -> Vec<TokenStream> {
    let mut namespaces = HashSet::new();
    for op in operators {
        let mut r = op.operatorType.qbNamespace.to_vec();
        r.push(op.operatorType.qbName.to_string());

        namespaces.insert(r);
    }

    namespaces
        .iter()
        .map(|r| {
            let initial_val = Ident::new(&r[0], Span::call_site());
            let ns_id = r
                .iter()
                .skip(1) // used as initial state for folding (assertion: must have at least one element!)
                .fold(quote!{ #initial_val }, |state, curr| {
                    let n = Ident::new(&curr, Span::call_site());
                    quote!{
                        #state::#n
                    }
                });

            quote!{
                use #ns_id;
            }
        })
        .collect()
}

fn generate_imports(operators: &Vec<Operator>) -> TokenStream {
    let app_namespaces = generate_app_namespaces(operators);

    quote!{
        use std::sync::mpsc::{Receiver, RecvError, Sender};
        use ohua_runtime::run_ohua;

        #(#app_namespaces)*
    }
}

pub fn generate_code(compiled: &OhuaData) -> TokenStream {
    let header_code = generate_imports(&compiled.graph.operators);
    let arc_code = generate_arcs(&compiled);
    let op_code = generate_sfns(&compiled);

    quote!{
        {
            #header_code

            #arc_code

            #op_code

            run_ohua(tasks)
        }
    }
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

    fn producer_consumer(prod: OperatorType, con: OperatorType, out_idx: i32) -> OhuaData {
        OhuaData {
            graph: DFGraph {
                operators: vec![
                    Operator {
                        operatorId: 0,
                        operatorType: prod,
                    },
                    Operator {
                        operatorId: 1,
                        operatorType: con,
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
                            index: out_idx,
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
        }
    }

    #[test]
    fn sf_code_gen() {
        let compiled = producer_consumer(
            OperatorType {
                qbNamespace: vec!["ns1".to_string()],
                qbName: "some_sfn".to_string(),
                func: "none".to_string(),
                op_type: OpType::SfnWrapper,
            },
            OperatorType {
                qbNamespace: vec!["ns2".to_string()],
                qbName: "some_other_sfn".to_string(),
                func: "none".to_string(),
                op_type: OpType::SfnWrapper,
            },
            -1,
        );

        let generated_imports = generate_imports(&compiled.graph.operators).to_string();
        println!(
            "\nGenerated code for imports:\n{}\n",
            &(generated_imports.replace(";", ";\n"))
        );
        assert!("use std :: sync :: mpsc :: { Receiver , Sender } ; use ohua_runtime :: run_ohua ; use ns1 :: some_sfn ; use ns2 :: some_other_sfn ;" == generated_imports);

        let generated_arcs = generate_arcs(&compiled).to_string();
        println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
        assert!(
            "let ( sf_0_out_0__sf_1_in_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;"
                == generated_arcs
        );

        let generated_sfns = generate_sfns(&compiled).to_string();
        println!(
            "Generated code for sfns:\n{}\n",
            &(generated_sfns.replace(";", ";\n"))
        );
        assert!("let mut tasks : Vec < Box < Fn ( ) -> ( ) + Send + 'static >> = Vec :: new ( ) ; tasks . push ( Box :: new ( move || { let r = some_sfn ( ) ; sf_0_out_0__sf_1_in_0 . send ( r ) . unwrap ( ) ; } ) ) ; tasks . push ( Box :: new ( move || { loop { let r = some_other_sfn ( sf_1_in_0 . recv ( ) ? ) ; } } ) ) ;" == generated_sfns);
    }

    #[test]
    fn op_code_gen() {
        let compiled = producer_consumer(
            OperatorType {
                qbNamespace: vec!["ns1".to_string()],
                qbName: "some_op".to_string(),
                func: "none".to_string(),
                op_type: OpType::OhuaOperator("whatever".to_string()),
            },
            OperatorType {
                qbNamespace: vec!["ns2".to_string()],
                qbName: "some_other_op".to_string(),
                func: "none".to_string(),
                op_type: OpType::OhuaOperator("whatever".to_string()),
            },
            0,
        );

        let generated_arcs = generate_arcs(&compiled).to_string();
        println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
        assert!(
            "let ( sf_0_out_0__sf_1_in_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;"
                == generated_arcs
        );

        let generated_ops = generate_ops(&compiled).to_string();
        println!(
            "Generated code for ops:\n{}\n",
            &(generated_ops.replace(";", ";\n"))
        );
        assert!("tasks . push ( || { loop { some_op ( & sf_0_out_0__sf_1_in_0 ) ; } } ) ; tasks . push ( || { loop { some_other_op ( & sf_1_in_0 ) ; } } ) ;" == generated_ops);
    }

    // {"graph":
    //   {"operators":[{"id":1,"type":{"namespace":["addition"],"name":"produce"}},
    //                 {"id":2,"type":{"namespace":["addition"],"name":"consume"}}],
    //    "arcs":[{"target":{"operator":2,"index":0},"source":{"type":"local","val":{"operator":1,"index":-1}}}],
    //    "return_arc":{"operator":2,"index":-1}
    //   },
    // #[test]
    // fn sf_code_gen() {
    //     let compiled = producer_consumer(OperatorType {
    //                                         qbNamespace: Vec::new(),
    //                                         qbName: "none".to_string(),
    //                                         func: "some_sfn".to_string(),
    //                                         op_type: OpType::SfnWrapper,
    //                                     },
    //                                     OperatorType {
    //                                         qbNamespace: Vec::new(),
    //                                         qbName: "none".to_string(),
    //                                         func: "some_other_sfn".to_string(),
    //                                         op_type: OpType::SfnWrapper,
    //                                     },
    //                                    -1);
    //
    //     let generated_arcs = generate_arcs(&compiled).to_string();
    //     println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
    //     // assert!("let ( sf_0_out_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;" == generated_arcs);
    //
    //     let generated_sfns = generate_sfns(&compiled).to_string();
    //     println!("Generated code for sfns:\n{}\n", &(generated_sfns.replace(";", ";\n")));
    //     // assert!("let mut tasks = Vec :: new ( ) ; tasks . push ( || { loop { if true { let r = some_sfn ( ) ; send ( r , vec ! [ & sf_0_out_0 ] ) ; } else { } } } ) ; tasks . push ( || { loop { if true { let r = some_other_sfn ( sf_1_in_0 . recv ( ) . unwrap ( ) ) ; send ( r , vec ! [ ] ) ; } else { sf_1_in_0 . recv ( ) . unwrap ( ) ; } } } ) ;" == generated_sfns);
    // }

}
