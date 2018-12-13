#![allow(unused_doc_comments)]
use ohua_types::ArcIdentifier;
use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc::{Receiver, Sender};

use ohua_types::ValueType::{EnvironmentVal, LocalVal};
use ohua_types::{NodeType, Arcs, DirectArc, CompoundArc, StateArc, ArcSource, OhuaData, Operator, OperatorType, ValueType};

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::Expr;

fn get_op_id(val: &ValueType) -> &i32 {
    match val {
        ValueType::EnvironmentVal(i) => i,
        ValueType::LocalVal(i) => &(i.operator),
    }
}

fn get_num_inputs(op: &i32, arcs: &Vec<DirectArc>) -> usize {
    arcs.iter()
        .filter(|arc| &(arc.target.operator) == op)
        .count()
}

fn get_num_outputs(op: &i32, arcs: &Vec<DirectArc>) -> usize {
    arcs.iter()
        .filter(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => unimplemented!(),
            LocalVal(a_id) => &(a_id.operator) == op,
        })
        .count()
}

fn get_outputs(op: &i32, arcs: &Vec<DirectArc>) -> Vec<i32> {
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

fn get_out_arcs<'a>(op: &i32, arcs: &'a Vec<DirectArc>) -> Vec<&'a DirectArc> {
    let t = arcs
        .iter()
        .filter(|arc| match &(arc.source.val) {
            EnvironmentVal(i) => false,
            LocalVal(a_id) => &(a_id.operator) == op,
        })
        .collect();
    t
}

fn get_in_arcs<'a>(op: &i32, arcs: &'a Vec<DirectArc>) -> Vec<&'a DirectArc> {
    let t = arcs
        .iter()
        .filter(|arc| &(arc.target.operator) == op)
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
    let computed_idx = match &(op_spec.nodeType) {
        NodeType::FunctionNode => {
            assert!(idx == &-1);
            &0
        }
        NodeType::OperatorNode => {
            assert!(idx >= &0);
            idx
        }
    };

    format!("sf_{}_out_{}", op.to_string(), computed_idx.to_string())
}

fn generate_out_arc_var(arc: &DirectArc, ops: &Vec<Operator>) -> Ident {
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

/**
Generates the parameters for a call.
*/
fn generate_in_arcs_vec(
    op: &i32,
    arcs: &Vec<DirectArc>,
    algo_call_args: &Punctuated<Expr, Token![,]>,
) -> Vec<TokenStream> {
    let mut in_arcs = get_in_arcs(op, arcs);
    in_arcs.sort_by_key(|a| a.target.index);

    in_arcs
        .iter()
        .filter(|arc| arc.target.index != -1)
        .map(|a| match a.source.val {
            EnvironmentVal(i) => algo_call_args
                .iter()
                .nth(i as usize)
                .expect(&format!("Invariant broken! {}, {}", i, algo_call_args.len()).to_string())
                .into_token_stream(),
            LocalVal(ref arc) => {
                generate_var_for_in_arc(&a.target.operator, &a.target.index).into_token_stream()
            }
        })
        .collect()
}

fn generate_out_arcs_vec(op: &i32, arcs: &Vec<DirectArc>, ops: &Vec<Operator>) -> Vec<Ident> {
    get_out_arcs(&op, &arcs)
        .iter()
        .map(|arc| generate_out_arc_var(arc, ops))
        .collect()
}

pub fn generate_arcs(compiled: &OhuaData) -> TokenStream {
    let arcs: Vec<&DirectArc> = compiled
        .graph
        .arcs
        .direct
        .iter()
        .filter(|a| filter_env_arc(&a))
        .collect();
    let outs = arcs
        .iter()
        .map(|arc| generate_out_arc_var(&arc, &(compiled.graph.operators)));
    let ins = arcs
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
        (match o.nodeType {
            NodeType::OperatorNode => true,
            _ => false,
        })
    });
    let op_codes: Vec<TokenStream> = ops
        .map(|op| {
            let mut call_args =
                generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs.direct), &Punctuated::new()); // ops can never have EnvArgs -> invariant broken
            let out_arcs = generate_out_arcs_vec(
                &(op.operatorId),
                &(compiled.graph.arcs.direct),
                &(compiled.graph.operators),
            );

            if out_arcs.len() > 0 {
                let c = out_arcs.iter().map(ToTokens::into_token_stream);
                call_args.extend(c);
            } else if op.operatorId == compiled.graph.return_arc.operator {
                // the return_arc is the output port
                call_args.push(quote!{ result_snd });
            }

            let op_name = get_call_reference(&op.operatorType);

            if call_args.len() > 0 {
                quote!{
                    loop{
                        #op_name(#(&#call_args),*)?;
                    }
                }
            } else {
                quote!{ #op_name() }
            }
        })
        .collect();

    quote!{
        #(tasks.push(Box::new(move || { #op_codes })); )*
    }
}

fn filter_env_arc(arc: &DirectArc) -> bool {
    match arc.source.val {
        EnvironmentVal(_) => false,
        _ => true,
    }
}

pub fn generate_sfns(
    compiled: &OhuaData,
    algo_call_args: &Punctuated<Expr, Token![,]>,
) -> TokenStream {
    let sfns = compiled
        .graph
        .operators
        .iter()
        .filter(|&o| match o.nodeType {
            NodeType::FunctionNode => true,
            _ => false,
        });
    let sf_codes: Vec<TokenStream> = sfns
        .map(|op| {
            let mut in_arcs =
                generate_in_arcs_vec(&(op.operatorId), &(compiled.graph.arcs.direct), algo_call_args);
            let orig_in_arcs = get_in_arcs(&(op.operatorId), &(compiled.graph.arcs.direct));
            let mut zipped_in_arcs: Vec<(&&DirectArc, TokenStream)> =
                orig_in_arcs.iter().zip(in_arcs.drain(..)).collect();

            // determine if cloning is necessary and apply it if so
            let mut seen_env_arcs = HashMap::new();
            let mut seen_local_arc = false;
            for pos in 0..zipped_in_arcs.len() {
                if let EnvironmentVal(x) = zipped_in_arcs[pos].0.source.val {
                    if let Some(old_pos) = seen_env_arcs.insert(x, pos) {
                        // the value is present, clone the old one
                        let old_ident = zipped_in_arcs[old_pos].1.clone();
                        zipped_in_arcs[old_pos].1 = quote!{ #old_ident.clone() };
                    }
                } else {
                    seen_local_arc = true;
                }
            }

            // necessary workaround to add cloning for non-"env arc only" operators where they are used in a loop
            if seen_local_arc {
                for (_, index) in seen_env_arcs {
                    let old_ident = zipped_in_arcs[index].1.clone();
                    zipped_in_arcs[index].1 = quote!{ #old_ident.clone() };
                }
            }

            let out_arcs = generate_out_arcs_vec(
                &op.operatorId,
                &(compiled.graph.arcs.direct),
                &(compiled.graph.operators),
            );

            let sf = get_call_reference(&op.operatorType);
            // let arcs = in_arcs.clone(); // can't reuse var in quote!
            let r = Ident::new(&"r", Span::call_site());
            let send = generate_send(
                &r,
                &out_arcs,
                &op.operatorId,
                &compiled.graph.return_arc.operator,
            );

            let drain_arcs: Vec<TokenStream> = zipped_in_arcs
                .iter()
                .filter(|(arc, _)| filter_env_arc(&arc))
                .map(|(_, t)| t.clone())
                .collect();
            let num_input_arcs = drain_arcs.len();
            // let drain_inputs = quote!{ #(#drain_arcs.recv()?;)* };

            let call_args: Vec<TokenStream> = zipped_in_arcs
                .iter()
                .map(|(orig_arc, code)| match orig_arc.source.val {
                    EnvironmentVal(_) => code.clone().clone(),
                    LocalVal(_) => quote!{ #code.recv()? },
                })
                .collect();
            let call_code = quote!{ #sf( #(#call_args),* ) };
            let sfn_code = quote!{ let #r = #call_code; #send };

            if num_input_arcs > 0 {
                quote!{ loop { #sfn_code } }
            } else {
                quote!{ #sfn_code; Ok(()) }
            }
        })
        .collect();

    quote!{
        let mut tasks: Vec<Box<FnBox() -> Result<(), RunError> + Send + 'static>> = Vec::new();
        #(tasks.push(Box::new(move || { #sf_codes })); )*
    }
}

fn generate_send(r: &Ident, outputs: &Vec<Ident>, op: &i32, final_op: &i32) -> TokenStream {
    // option 1: we could borrow here and then it would fail if somebody tries to write to val. (pass-by-ref)
    // option 2: clone (pass-by-val)
    // this is something that our knowledge base could be useful for: check if any of the predecessor.
    // requires a mutable reference. if so then we need a clone for this predecessor.
    // borrowing across channels does not seem to work. how do I make a ref read-only in Rust? is this possible at all?
    match outputs.len() {
        0 => {
            if op == final_op {
                quote!{ result_snd.send(#r)?; }
            } else {
                quote!{} // drop
            }
        }
        1 => {
            let o = &outputs[0];
            quote!{ #o.send(#r)? }
        }
        _ => {
            let results: Vec<Ident> = outputs.iter().map(|x| r.clone()).collect();
            quote!{
                #(#outputs.send(#results.clone())?);*;
            }
        }
    }
}

fn generate_app_namespaces(operators: &Vec<Operator>) -> Vec<TokenStream> {
    let mut namespaces = BTreeSet::new();
    for op in operators {
        // ignore imports in the root
        if op.operatorType.qbNamespace.is_empty() {
            continue;
        }
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
        use std::sync::mpsc::{Receiver, Sender};
        use std::boxed::FnBox;
        use ohua_runtime::*;

        #(#app_namespaces)*
    }
}

fn generate_call_to_ctrl(ctrl_in: Ident, ins: Vec<Ident>, outs: Vec<Ident>) -> TokenStream {
    let num_args = ins.len();
    assert!(ins.len() == outs.len());
    quote!{
        ctrl_#num_args(#ctrl_in, ( #(#ins),* ) , ( #(#outs),* ) )
    }
}

fn generate_ctrl_operator(num_args:isize) -> TokenStream {
    let vars_in:Vec<Ident> = (0..num_args).map(|arg_idx| {
                              Ident::new(&format!("var_in_{}", arg_idx.to_string()),
                                         Span::call_site()) })
                                          .collect();
    let vars_out:Vec<Ident> = (0..num_args).map(|arg_idx| {
                              Ident::new(&format!("var_out_{}", arg_idx.to_string()),
                                         Span::call_site()) })
                                           .collect();
    let vars:Vec<Ident> = (0..num_args).map(|arg_idx| {
                              Ident::new(&format!("var_{}", arg_idx.to_string()),
                                         Span::call_site()) })
                                       .collect();

    quote!{
        //
        // The ctrl operator needs to be stateful but can not define its own state.
        // The state of the operator would have to capture the data retrieved from the
        // arcs of the vars. But what would be their type???
        // For a generic backend, we don't ever deal with types. We rely solely on the
        // type inference mechanisms of the target language.
        // As such, we can not write or even generate a function that creates this state.
        // THIS IS A GENERAL INSIGHT: state in operators (which are meant to be polymorph with
        // respect to the data in the graph) can never contain anything that is related to the
        // data in the graph!
        // As such, the only way to write such an operator is using tail recursion as shown below.
        //
        let ctrl_#num_args = |ctrl_inp, #(#vars_in,)* #(#vars_out),*| {
            let (renew_next_time, count) = ctrl_inp.recv()?;
            let vars = ( #(#vars_in.recv().unwrap()),* );
            for _ in 0..count {
                #(#vars_out.send(#vars.clone()).unwrap();)*
            }
            ctrl_#num_args(ctrl_inp, ins, outs, renew_next_time, vars)
        };

        let ctrl_#num_args = |ctrl_inp, old_vars, vars_in, vars_out, renew, old_vars| {
            let (renew_next_time, count) = ctrl_inp.recv()?;
            let vars = if renew {
                            ( #(#vars_in.recv().unwrap()),* );
                       } else {
                           // reuse the captured vars
                           old_vars
                       };
            for _ in 0..count {
               #(#vars_out.send(#vars.clone()).unwrap();)*
            }
            ctrl_#num_args(ctrl_inp, ins, outs, renew_next_time, vars)
        }
    }
}

fn generate_ctrls(compiled_algo: &mut OhuaData) -> TokenStream {
    let code =
        compiled_algo.graph.operators
             .iter()
             .filter(|op| {
                 op.operatorType.qbNamespace == vec!["lang"] &&
                 op.operatorType.qbName.as_str() ==  "ctrl"
             })
             .map(|op|{
                let num_args = get_num_inputs(op);
                generate_ctrl_operator(num_args-1)
             })
             .collect();

    quote!{
        #(#code)*
    }
}

// fn handle_scope_operator(compiled_algo: &mut OhuaData) -> TokenStream {
//     let mut scope_functions: Vec<TokenStream> = Vec::new();
//     for operator in &mut compiled_algo.graph.operators {
//         if operator.operatorType.qbName == "scope"
//             && operator.operatorType.qbNamespace == vec!["ohua_runtime", "lang"]
//         {
//             // remove the namespace prefix, as we will generate this function
//             operator.operatorType.qbNamespace = Vec::with_capacity(0);
//
//             // generate the appropriate scope function
//             let func_name = format!("scope{n}", n = scope_functions.len());
//             let fn_name = Ident::new(&func_name, Span::call_site());
//
//             // num_inputs minus one, because we do not want the control arc as input
//             let param_input_iterator =
//                 0..(get_num_inputs(&operator.operatorId, &compiled_algo.graph.arcs) - 1);
//
//             let type_params: Vec<Ident> =
//                 param_input_iterator.clone().map(|x| Ident::new(&format!("T{}", x), Span::call_site())).collect();
//
//             let params_ret: Vec<Ident> = param_input_iterator
//                 .map(|x| Ident::new(&format!("t{n}", n = x), Span::call_site()))
//                 .collect();
//
//             let mut params_ret2 = params_ret.clone();
//             let mut type_params2 = type_params.clone();
//             let params: Vec<TokenStream> = params_ret2.drain(..).zip(type_params2.drain(..)).map(|(id, ty)| quote!{#id: #ty}).collect();
//
//             // let params: Vec<Ident> = param_input_iterator.clone()
//             //     .map(|x| Ident::new(&format!("t{n}: T{n}", n = x), Span::call_site()))
//             //     .collect();
//             let type_params_ret = type_params.clone();
//
//             // generate the actual scope function
//             let scope_fn = quote!{
//                 fn #fn_name<#(#type_params),*>(#(#params),*) -> (#(#type_params_ret),*) {
//                     (#(#params_ret),*)
//                 }
//             };
//
//             // add it to the structure
//             scope_functions.push(scope_fn);
//             operator.operatorType.qbName = func_name;
//
//             for arc in compiled_algo.graph.arcs.iter_mut() {
//                 if let ValueType::LocalVal(ref mut src) = arc.source.val {
//                     if src.operator == operator.operatorId {
//                         src.index = -1;
//                     }
//                 }
//             }
//         }
//     }
//     quote!{
//         #(
//             #scope_functions
//         )*
//     }
// }

/// Change the operator types for `smapFun` and `oneToN` from SfnWrapper to OhuaOperator
// fn change_operator_types(compiled_algo: &mut OhuaData) {
//     for op in &mut compiled_algo.graph.operators {
//         if op.operatorType.qbNamespace == vec!["ohua_runtime", "lang"] {
//             match op.operatorType.qbName.as_str() {
//                 "oneToN" |
//                 "smapFun" |
//                 "collect" |
//                 "select" => {
//                     op.operatorType.op_type = OpType::OhuaOperator("whatever".into());
//                     for arc in compiled_algo.graph.arcs.iter_mut() {
//                         if let ValueType::LocalVal(ref mut src) = arc.source.val {
//                             if src.operator == op.operatorId {
//                                 src.index = 0;
//                             }
//                         }
//                     }
//                 }
//                 "bool" => {
//                     op.operatorType.op_type = OpType::OhuaOperator("whatever".into());
//                 }
//                 _ => (),
//             }
//         }
//     }
// }

fn handle_environment_arcs(compiled_algo: &mut OhuaData) {
    // special-casing of zero env-arcs as they still have a main arity of one.
    // Hotfix until ohua-dev/ohuac#16 is fixed
    if compiled_algo
        .graph
        .arcs
        .iter()
        .filter(|a| {
            if let ValueType::EnvironmentVal(_) = a.source.val {
                true
            } else {
                false
            }
        })
        .count()
        == 0
    {
        return;
    }

    // find starting number for next operator
    let new_op_id_base: i32 = compiled_algo.graph.operators.iter().fold(0, |acc, op| {
        if op.operatorId > acc {
            op.operatorId
        } else {
            acc
        }
    }) + 1;

    for i in 0..(compiled_algo.mainArity) {
        // add a new operator per environment arc
        compiled_algo.graph.operators.push(Operator {
            operatorId: new_op_id_base + i,
            operatorType: OperatorType {
                qbNamespace: vec!["ohua_runtime".into(), "lang".into()],
                qbName: "id".into(),
            },
            nodeType: NodeType::FunctionNode,
        });

        // reroute any env-arcs with matching id
        for arc in &mut compiled_algo.graph.arcs.direct {
            if let ValueType::EnvironmentVal(env_id) = arc.source.val {
                if i == env_id {
                    arc.source = ArcSource {
                        s_type: "local".into(),
                        val: ValueType::LocalVal(ArcIdentifier {
                            operator: new_op_id_base + i,
                            index: -1,
                        }),
                    }
                }
            }
        }

        compiled_algo.graph.arcs.direct.push(DirectArc {
            target: ArcIdentifier {
                operator: new_op_id_base + i,
                index: 0,
            },
            source: ArcSource {
                s_type: "env".into(),
                val: ValueType::EnvironmentVal(i),
            },
        });
    }
}

pub fn generate_code(
    compiled_algo: &mut OhuaData,
    algo_call_args: &Punctuated<Expr, Token![,]>,
) -> TokenStream {
    let ctrl_code = generate_ctrls(compiled_algo);
    handle_environment_arcs(compiled_algo);
    let header_code = generate_imports(&compiled_algo.graph.operators);
    let arc_code = generate_arcs(&compiled_algo);
    let sf_code = generate_sfns(&compiled_algo, algo_call_args);
    let op_code = generate_ops(&compiled_algo);

    // Macro hygiene: I can create a variable here and use it throughout the whole call-site of this
    // macro because quote! has Span:call_site() -> call site = call site of the macro!
    // https://github.com/dtolnay/quote
    // https://docs.rs/proc-macro2/0.4/proc_macro2/struct.Span.html#method.call_site
    quote!{
        {
            #header_code

            #ctrl_code

            #arc_code
            let (result_snd, result_rcv) = std::sync::mpsc::channel();

            #sf_code

            #op_code

            run_tasks(tasks);
            result_rcv.recv().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use parse::tests::parse_call;
    use syn::ExprPath;
    use syn::Path;
    use syn::PathSegment;

    use std::sync::mpsc;

    use ohua_types::ArcIdentifier;
    use ohua_types::ArcSource;
    use ohua_types::DFGraph;
    use ohua_types::OhuaData;
    use ohua_types::Operator;
    use ohua_types::OperatorType;
    use ohua_types::ValueType;

    fn producer_consumer(prod: OperatorType, prod_t: NodeType,
                         con: OperatorType, con_t: NodeType, out_idx: i32) -> OhuaData {
        OhuaData {
            graph: DFGraph {
                operators: vec![
                    Operator {
                        operatorId: 0,
                        operatorType: prod,
                        nodeType: prod_t,
                    },
                    Operator {
                        operatorId: 1,
                        operatorType: con,
                        nodeType: con_t,
                    },
                ],
                arcs: Arcs {
                    direct: vec![Arc {
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
                    compound: vec![],
                    state: vec![],
                },
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
            },
            NodeType::FunctionNode,
            OperatorType {
                qbNamespace: vec!["ns2".to_string()],
                qbName: "some_other_sfn".to_string(),
            },
            NodeType::FunctionNode,
            -1,
        );

        let generated_imports = generate_imports(&compiled.graph.operators).to_string();
        // println!(
        //     "\nGenerated code for imports:\n{}\n",
        //     &(generated_imports.replace(";", ";\n"))
        // );
        assert!("use std :: sync :: mpsc :: { Receiver , Sender } ; use std :: boxed :: FnBox ; use ohua_runtime :: * ; use ns1 :: some_sfn ; use ns2 :: some_other_sfn ;" == generated_imports);

        let generated_arcs = generate_arcs(&compiled).to_string();
        // println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
        assert!(
            "let ( sf_0_out_0__sf_1_in_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;"
                == generated_arcs
        );

        let generated_sfns = generate_sfns(&compiled, &Punctuated::new()).to_string();
        // println!(
        //     "Generated code for sfns:\n{}\n",
        //     &(generated_sfns.replace(";", ";\n"))
        // );
        assert!("let mut tasks : Vec < Box < FnBox ( ) -> Result < ( ) , RunError > + Send + 'static >> = Vec :: new ( ) ; tasks . push ( Box :: new ( move || { let r = some_sfn ( ) ; sf_0_out_0__sf_1_in_0 . send ( r ) ? ; Ok ( ( ) ) } ) ) ; tasks . push ( Box :: new ( move || { loop { let r = some_other_sfn ( sf_1_in_0 . recv ( ) ? ) ; result_snd . send ( r ) ? ; } } ) ) ;" == generated_sfns);
    }

    #[test]
    fn op_code_gen() {
        let compiled = producer_consumer(
            OperatorType {
                qbNamespace: vec!["ns1".to_string()],
                qbName: "some_op".to_string(),
            },
            NodeType::OperatorNode,
            OperatorType {
                qbNamespace: vec!["ns2".to_string()],
                qbName: "some_other_op".to_string(),
            },
            NodeType::OperatorNode,
            0,
        );

        let generated_arcs = generate_arcs(&compiled).to_string();
        // println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
        assert!(
            "let ( sf_0_out_0__sf_1_in_0 , sf_1_in_0 ) = std :: sync :: mpsc :: channel ( ) ;"
                == generated_arcs
        );

        let generated_ops = generate_ops(&compiled).to_string();
        // println!(
        //     "Generated code for ops:\n{}\n",
        //     &(generated_ops.replace(";", ";\n"))
        // );
        assert!("tasks . push ( || { loop { some_op ( & sf_0_out_0__sf_1_in_0 ) ; } } ) ; tasks . push ( || { loop { some_other_op ( & sf_1_in_0 ) ; } } ) ;" == generated_ops);
    }

    #[test]
    fn env_args_code_gen() {
        let compiled = OhuaData {
            graph: DFGraph {
                operators: vec![Operator {
                    operatorId: 0,
                    operatorType: OperatorType {
                        qbNamespace: vec!["ns1".to_string()],
                        qbName: "some_sfn".to_string(),
                    },
                    nodeType: NodeType::FunctionNode,
                }],
                arcs: Arcs {
                    direct: vec![DirectArc {
                        target: ArcIdentifier {
                            operator: 0,
                            index: 0,
                        },
                        source: ArcSource {
                            s_type: "".to_string(),
                            val: ValueType::EnvironmentVal(0),
                        },
                    }],
                    compound: vec![],
                    state: vec![],
                },
                return_arc: ArcIdentifier {
                    operator: 1,
                    index: -1,
                },
                input_targets: Vec::new(),
            },
            mainArity: 1,
            sfDependencies: Vec::new(),
        };

        // let c = quote!{ some_algo(arg1) };
        // let b = c.into();
        // let a = TokenStream::new().into();
        // let (_, call_args) = parse_ohua_call(a, b);
        // above does not work: panicked at 'procedural macro API is used outside of a procedural macro', libproc_macro/lib.rs:1314:13

        let (_, call_args) = parse_call("some_algo(arg1)");

        let generated_arcs = generate_arcs(&compiled).to_string();
        // println!("\nGenerated code for arcs:\n{}\n", &generated_arcs);
        assert!("" == generated_arcs);

        let generated_sfns = generate_sfns(&compiled, &call_args).to_string();
        // println!(
        //     "Generated code for sfns:\n{}\n",
        //     &(generated_sfns.replace(";", ";\n"))
        // );
        assert!("let mut tasks : Vec < Box < FnBox ( ) -> Result < ( ) , RunError > + Send + 'static >> = Vec :: new ( ) ; tasks . push ( Box :: new ( move || { let r = some_sfn ( arg1 ) ; ; Ok ( ( ) ) } ) ) ;" == generated_sfns);
    }
}
