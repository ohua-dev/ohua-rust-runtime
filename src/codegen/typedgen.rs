#![allow(unused_doc_comments)]
use std::sync::mpsc::{Receiver,Sender};

use ohua_types::{Arc, OhuaData, ValueType, OpType, Operator};
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

fn generate_in_arcs_vec(op: &i32, arcs: &Vec<Arc>) -> Vec<Ident> {
    let n = get_num_inputs(&op, &arcs);
    (0..n).map(|i| { Ident::new(&format!("sf_{}_in_{}", op.to_string(), i.to_string()),
                                Span::call_site()) }).collect()
}

fn generate_out_arcs_vec(op: &i32, arcs: &Vec<Arc>) -> Vec<Ident> {
    let n = get_num_outputs(&op, &arcs);
    // FIXME to pass them as normal arguments, we need an index!
    (0..n).map(|i| { Ident::new(&format!("sf_{}_in_{}", op.to_string(), i.to_string()),
                                Span::call_site()) }).collect()
}

pub fn generate_arcs(compiled: &OhuaData) -> TokenStream {
    let outs = compiled.graph.arcs.iter().map(|arc| {
        let op = get_op_id(&(arc.source.val));
        Ident::new(&format!("sf_{}_out", op), Span::call_site())
    });
    let ins = compiled.graph.arcs.iter().map(|arc| {
        Ident::new(&format!("sf_{}_in_{}", arc.target.operator.to_string(), arc.target.index.to_string()),
                   Span::call_site())
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
        let in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
        let out_arcs = generate_out_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
        let op_name = Ident::new(&op.operatorType.func, Span::call_site());
        quote!{
            #op_name(#(#in_arcs),* #(,#out_arcs)*);
        }
    }).collect();

    quote!{
        #(tasks.push(|| { #op_codes }); )*
    }
}

pub fn generate_sfns(compiled: &OhuaData) -> TokenStream {
    let sfns = compiled.graph.operators.iter().filter(|&o| { match o.operatorType.op_type {
            OpType::SfnWrapper => true,
            _ => false,
    }});
    let sf_codes : Vec<TokenStream> = sfns.map(|op| {
        let in_arcs = generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs));
        let out_arc = Ident::new(&format!("sf_{}_out", op.operatorId.to_string()), Span::call_site());
        let sf = Ident::new(&op.operatorType.func, Span::call_site());
        // TODO turn this into a loop and exit when an error on the input channel occurs
        // TODO use a vector of outputs (create a normal output function because all the type related stuff happens until the sf was executed)
        // TODO implement control information (control arcs have target index set to -1)
        quote!{
            let r = #sf( #(#in_arcs.recv().unwrap()),* );
            #out_arc.send(r).unwrap();
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
         let generated_arcs = generate_arcs(&compiled).to_string();
         println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
         assert!("let ( sf_0_out , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;" == generated_arcs);

         let generated_sfns = generate_sfns(&compiled).to_string();
         println!("Generated code for sfns:\n{}\n", &(generated_sfns.replace(";", ";\n")));
         assert!("let mut tasks = Vec :: new ( ) ; tasks . push ( || { let r = some_sfn ( ) ; sf_0_out . send ( r ) . unwrap ( ) ; } ) ; tasks . push ( || { let r = some_other_sfn ( sf_1_in_0 . recv ( ) . unwrap ( ) ) ; sf_1_out . send ( r ) . unwrap ( ) ; } ) ;" == generated_sfns);
     }
 }
