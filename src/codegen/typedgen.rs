#![allow(unused_doc_comments)]
use std::sync::mpsc::{Receiver,Sender};

use ohua_types::{Arc, OhuaData, ValueType, OpType, Operator, ArcSource};
use ohua_types::ValueType::{EnvironmentVal, LocalVal};

use proc_macro2::{Ident,Span,TokenStream};

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
        .filter(|arc| { match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => &(a_id.operator) == op,
        }})
        .count()
}

fn get_out_index_from_source(src: &ArcSource) -> &i32 {
    match src.val {
        EnvironmentVal(ref i) => i,
        LocalVal(ref arc_id) => &arc_id.index,
    }
}

fn generate_var_for_out_arc(op: &i32, idx: &i32, ops: &Vec<Operator>) -> Ident {
    let op_spec = ops.iter()
                     .find(|o| &(o.operatorId) == op)
                     // FIXME This may fail for environment variables.
                     .expect(&format!("Ohua compiler invariant broken: Operator not registered: {}", op));
     let computed_idx = match &(op_spec.operatorType.op_type) {
         OpType::SfnWrapper => {
             assert!(idx == &-1);
             &0
         },
         OpType::OhuaOperator(_) => {
             assert!(idx >= &0);
             idx
         },
     };
    Ident::new(&format!("sf_{}_out_{}", op.to_string(), computed_idx.to_string()),
               Span::call_site())
}

fn generate_var_for_in_arc(op: &i32, idx: &i32) -> Ident {
    assert!(idx > &-2);
    let index = match idx {
                    -1 => "ctrl".to_string(),
                    _ => idx.to_string(),
                };
    Ident::new(&format!("sf_{}_in_{}", op.to_string(), index),
               Span::call_site())
}

fn generate_in_arcs_vec(op: &i32, arcs: &Vec<Arc>) -> Vec<Ident> {
    let n = get_num_inputs(&op, &arcs);
    // TODO handle control arcs. (control arc := target_idx = -1)
    (0..n).map(|i| { generate_var_for_in_arc(op, &(i as i32)) }).collect()
}

fn generate_out_arcs_vec(op: &i32, arcs: &Vec<Arc>, ops: &Vec<Operator>) -> Vec<Ident> {
    let n = get_num_outputs(&op, &arcs);
    (0..n).map(|i| { generate_var_for_out_arc(op, &(i as i32), ops) }).collect()
}

pub fn generate_arcs(compiled: &OhuaData) -> TokenStream {
    let outs = compiled.graph.arcs.iter().map(|arc| {
        let op = get_op_id(&(arc.source.val));
        generate_var_for_out_arc(&op,
                                 get_out_index_from_source(&arc.source),
                                 &(compiled.graph.operators))
    });
    let ins = compiled.graph.arcs.iter().map(|arc| {
        generate_var_for_in_arc(&(arc.target.operator), &(arc.target.index))
    });

    quote!{
        #(let (#outs, #ins) = std::sync::mpsc::channel();)*
    }
}

pub fn generate_ops(compiled: &OhuaData) -> TokenStream {
    let ops = compiled.graph.operators.iter().filter(|o| (match o.operatorType.op_type {
            OpType::OhuaOperator(_) => true,
            _ => false,
    }));
    let op_codes : Vec<TokenStream> = ops.map(|op| {
        let mut in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
        let mut out_arcs = generate_out_arcs_vec(&(op.operatorId), &(compiled.graph.arcs), &(compiled.graph.operators));
        let op_name = Ident::new(&op.operatorType.func, Span::call_site());

        in_arcs.append(&mut out_arcs);
        assert!(find_control_input(&(op.operatorId), &compiled.graph.arcs).is_none());
        
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
    }).collect();

    quote!{
        #(tasks.push(|| { #op_codes }); )*
    }
}

fn find_control_input(op: &i32, arcs: &Vec<Arc>) -> Option<Ident> {
    arcs.into_iter()
        .find(|arc| { &(arc.target.operator) == op && arc.target.index == -1 } )
        .map(|ctrl_arc| { generate_var_for_in_arc(&(ctrl_arc.target.operator), &(ctrl_arc.target.index)) } )
}

pub fn generate_sfns(compiled: &OhuaData) -> TokenStream {
    let sfns = compiled.graph.operators.iter().filter(|&o| { match o.operatorType.op_type {
            OpType::SfnWrapper => true,
            _ => false,
    }});
    let sf_codes : Vec<TokenStream> = sfns.map(|op| {
        let in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
        let out_arc = generate_var_for_out_arc(&op.operatorId, &-1, &(compiled.graph.operators));
        let sf = Ident::new(&op.operatorType.func, Span::call_site());
        // TODO use a vector of outputs (create a normal output function because all the type related stuff happens until the sf was executed)
        let ctrl_port = find_control_input(&(op.operatorId), &compiled.graph.arcs);
        let ctrl = match ctrl_port {
                None => quote! {true},
                Some(p) => quote!{#p.recv().unwrap()},
            };
        let arcs = in_arcs.clone(); // can't reuse var in quote!
        quote!{
            loop {
                if #ctrl {
                    let r = #sf( #(#in_arcs.recv().unwrap()),* );
                    #out_arc.send(r).unwrap();
                } else {
                    // just drain and skip the call
                    #(#arcs.recv().unwrap();)*
                }
            }
        }
    }).collect();

    quote!{
        let mut tasks = Vec::new();
        #(tasks.push(|| { #sf_codes }); )*
    }
}

pub fn generate_code(compiled: &OhuaData) -> TokenStream {
    // generate imports
    // FIXME use quote instead!
    // let namespaces = codegen::wrappers::analyze_namespaces(&compiled);
    // let imports = namespaces
    //     .iter()
    //     .fold(String::new(), |acc, ref x| acc + "use " + x + ";\n");

    let arc_code = generate_arcs(&compiled);
    let op_code = generate_sfns(&compiled);

    quote!{
        {
            // #header_code

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
         let compiled = producer_consumer(OperatorType {
                                             qbNamespace: Vec::new(),
                                             qbName: "none".to_string(),
                                             func: "some_sfn".to_string(),
                                             op_type: OpType::SfnWrapper,
                                         },
                                         OperatorType {
                                             qbNamespace: Vec::new(),
                                             qbName: "none".to_string(),
                                             func: "some_other_sfn".to_string(),
                                             op_type: OpType::SfnWrapper,
                                         },
                                        -1);

         let generated_arcs = generate_arcs(&compiled).to_string();
         println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
         assert!("let ( sf_0_out_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;" == generated_arcs);

         let generated_sfns = generate_sfns(&compiled).to_string();
         println!("Generated code for sfns:\n{}\n", &(generated_sfns.replace(";", ";\n")));
         assert!("let mut tasks = Vec :: new ( ) ; tasks . push ( || { loop { if true { let r = some_sfn ( ) ; sf_0_out_0 . send ( r ) . unwrap ( ) ; } else { } } } ) ; tasks . push ( || { loop { if true { let r = some_other_sfn ( sf_1_in_0 . recv ( ) . unwrap ( ) ) ; sf_1_out_0 . send ( r ) . unwrap ( ) ; } else { sf_1_in_0 . recv ( ) . unwrap ( ) ; } } } ) ;" == generated_sfns);
     }

     #[test]
     fn op_code_gen() {
         let compiled = producer_consumer(OperatorType {
                                             qbNamespace: Vec::new(),
                                             qbName: "none".to_string(),
                                             func: "some_op".to_string(),
                                             op_type: OpType::OhuaOperator("whatever".to_string()),
                                         },
                                         OperatorType {
                                             qbNamespace: Vec::new(),
                                             qbName: "none".to_string(),
                                             func: "some_other_op".to_string(),
                                             op_type: OpType::OhuaOperator("whatever".to_string()),
                                         },
                                        0);

         let generated_arcs = generate_arcs(&compiled).to_string();
         println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
         assert!("let ( sf_0_out_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;" == generated_arcs);

         let generated_ops = generate_ops(&compiled).to_string();
         println!("Generated code for ops:\n{}\n", &(generated_ops.replace(";", ";\n")));
         assert!("tasks . push ( || { loop { some_op ( & sf_0_out_0 ) ; } } ) ; tasks . push ( || { loop { some_other_op ( & sf_1_in_0 ) ; } } ) ;" == generated_ops);
     }
 }
