#![allow(unused_doc_comments)]
use backend_optimizations::run_backend_optimizations;
use lang::{generate_ctrl_operator, generate_nth};
use ohua_types::ArcSource::{Env, Local};
use ohua_types::Envs::*;
use ohua_types::*;

use std::collections::BTreeSet;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::Expr;

const OHUA_RUNTIME_NAMESPACE: [&str; 2] = ["ohua_runtime", "lang"];

fn is_runtime_op(op: &Operator) -> bool {
    op.operatorType.qbNamespace == OHUA_RUNTIME_NAMESPACE
}

fn get_op_id(val: &ArcSource) -> &i32 {
    match val {
        ArcSource::Env(e) => match e {
            EnvRefLit { content: i } => i,
            _ => unimplemented!("get_op_id -> other literals"),
        },
        ArcSource::Local(i) => &(i.operator),
    }
}

fn get_num_inputs(op: &i32, arcs: &Vec<DirectArc>) -> usize {
    arcs.iter()
        .filter(|arc| &(arc.target.operator) == op)
        .count()
}

// TODO: Is this still needed?
#[allow(dead_code)]
fn get_num_outputs(op: &i32, arcs: &Vec<DirectArc>) -> usize {
    arcs.iter()
        .filter(|arc| match &(arc.source) {
            Env(_) => unimplemented!("get_num_outputs -> env args"),
            Local(a_id) => &(a_id.operator) == op,
        })
        .count()
}

// TODO: Is this still needed?
#[allow(dead_code)]
fn get_outputs(op: &i32, arcs: &Vec<DirectArc>) -> Vec<i32> {
    let mut t: Vec<i32> = arcs
        .iter()
        .filter(|arc| match &(arc.source) {
            Env(_) => unimplemented!("get_outputs (1) -> env args"),
            Local(a_id) => &(a_id.operator) == op,
        })
        .map(|arc| match &(arc.source) {
            Env(_) => unimplemented!("get_outputs (2) -> env args"),
            Local(a_id) => a_id.index,
        })
        .collect();
    t.sort();
    t
}

pub fn get_out_arcs<'a>(op: &i32, arcs: &'a Vec<DirectArc>) -> Vec<&'a DirectArc> {
    let t = arcs
        .iter()
        .filter(|arc| match &(arc.source) {
            Env(_) => false,
            Local(a_id) => &(a_id.operator) == op,
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
    match src {
        Env(ref e) => match e {
            EnvRefLit { content: ref i } => i,
            _ => unimplemented!("get_out_index_from_source -> other literals"),
        },
        Local(ref arc_id) => &arc_id.index,
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
            assert!(idx == &0);
            &0
        }
        NodeType::OperatorNode => {
            assert!(idx >= &0);
            idx
        }
    };

    format!("sf_{}_out_{}", op.to_string(), computed_idx.to_string())
}

fn make_out_arc_ident(out_port: &String, in_port: &Ident) -> Ident {
    /**
    This enforces the following invariant in Ohua/dataflow:
    An input port can only have one incoming arc. So it is enough to use the op-id and the input-port-id
    to generate a unique variable name.
    However, it is possible for one output port to have more than one outgoing arc.
    As such, using only the op-id in combination with the output-port-id will not work because
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

fn generate_out_arc_var(arc: &DirectArc, ops: &Vec<Operator>) -> Ident {
    let out_idx = get_out_index_from_source(&(arc.source));
    let src_op = get_op_id(&(arc.source));
    let out_port = generate_var_for_out_arc(src_op, out_idx, &ops);

    let in_port = generate_var_for_in_arc(&(arc.target.operator), &(arc.target.index));
    make_out_arc_ident(&out_port, &in_port)
}

fn generate_var_for_in_arc(op: &i32, idx: &i32) -> Ident {
    assert!(idx > &-2);
    // let index = match idx {
    //     -1 => "ctrl".to_string(),
    //     _ => idx.to_string(),
    // };
    Ident::new(
        &format!("sf_{}_in_{}", op.to_string(), idx),
        Span::call_site(),
    )
}

fn generate_send_var_for_state_arc(arc: &StateArc, ops: &Vec<Operator>) -> Ident {
    let out_idx = get_out_index_from_source(&(arc.source));
    let src_op = get_op_id(&(arc.source));
    let out_port = generate_var_for_out_arc(src_op, out_idx, &ops);

    Ident::new(
        &format!("{}__state", out_port.to_string()),
        Span::call_site(),
    )
}

fn generate_recv_var_for_state_arc(op: &i32) -> Ident {
    Ident::new(&format!("sf_{}_state", op.to_string()), Span::call_site())
}

/**
Generates the parameters for a call.
*/
#[allow(unreachable_patterns)]
fn generate_in_arcs_vec(
    op: &i32,
    node_type: &NodeType,
    arcs: &Vec<DirectArc>,
    algo_call_args: &Punctuated<Expr, Token![,]>,
) -> Vec<TokenStream> {
    let mut in_arcs = get_in_arcs(op, arcs);
    in_arcs.sort_by_key(|a| a.target.index);

    in_arcs
        .iter()
        .filter(|arc| arc.target.index != -1)
        .map(|a| match a.source {
            Env(ref e) => match e {
                EnvRefLit { content: i } => algo_call_args
                    .iter()
                    .nth(*i as usize)
                    .expect(
                        &format!(
                            "Invariant broken! Env arg idx: {}, Algo call args length: {}",
                            i,
                            algo_call_args.len()
                        )
                        .to_string(),
                    )
                    .into_token_stream(),
                NumericLit { content: num } => {
                    let n = Literal::i32_unsuffixed(*num);
                    match *node_type {
                        NodeType::FunctionNode => quote! { #n },
                        NodeType::OperatorNode => quote! { send_once(#n) },
                    }
                }
                UnitLit {} => {
                    match *node_type {
                        // FIXME make sure that this is the unitFn function!
                        NodeType::FunctionNode => quote! { Unit{} },
                        NodeType::OperatorNode => quote! { send_once(Unit{}) },
                    }
                }
                FunRefLit { contents: fn_ref } => {
                    // let f = syn::parse_str::<Path>(&fn_ref.0.function_name()).unwrap();
                    let f = get_call_reference(&fn_ref);
                    match *node_type {
                        NodeType::FunctionNode => quote! { #f },
                        NodeType::OperatorNode => quote! { send_once(#f) },
                    }
                }
                _ => unimplemented!("generate_in_arcs_vec -> other literals"),
            },
            Local(_) => {
                generate_var_for_in_arc(&a.target.operator, &a.target.index).into_token_stream()
            }
        })
        .collect()
}

// FIXME: Either include DeadArcs here or rewrite to Workaround arc
fn generate_out_arcs_vec<'a>(
    op: &i32,
    arcs: &'a Vec<DirectArc>,
    ops: &Vec<Operator>,
) -> Vec<(&'a DirectArc, Ident)> {
    // grab all out arcs and generate identifiers for them
    let all_out_arcs = get_out_arcs(&op, &arcs);
    let mut out_arcs: Vec<(&i32, (&'a DirectArc, Ident))> = all_out_arcs
        .iter()
        .map(|arc| {
            (
                get_out_index_from_source(&arc.source),
                (*arc, generate_out_arc_var(arc, ops)),
            )
        })
        .collect();

    // detect all output ports that serve more than one outgoing arc and bundle them,
    // create speacial identifiers for them
    let mut all_pairs = find_out_port_pairs(&arcs);
    let pairs: Vec<Vec<&'a DirectArc>> = all_pairs
        .drain(..)
        .filter(|arcs| match arcs[0].source {
            Local(ref a) => &a.operator == op,
            _ => false,
        })
        .collect();
    let mut pair_arcs: Vec<(&i32, (&'a DirectArc, Ident))> = pairs
        .iter()
        .map(|arcs| {
            (
                get_out_index_from_source(&arcs[0].source),
                (arcs[0], generate_pair_arc_var(arcs[0])),
            )
        })
        .collect();

    // replace all output arcs that share a port with the duplicating out arc
    for pair in pair_arcs.drain(..) {
        out_arcs.retain(|arc| get_out_index_from_source(&(arc.1).0.source) != pair.0);
        out_arcs.push(pair);
    }
    out_arcs.sort_unstable_by_key(|x| x.0);

    let res = out_arcs
        .drain(..)
        .unzip::<&i32, (&'a DirectArc, Ident), Vec<&i32>, Vec<(&'a DirectArc, Ident)>>()
        .1;

    res
}

/// Finds any output ports on an operator that serve multiple outgoing arcs and returns them,
/// bundled together in separate vectors.
fn find_out_port_pairs<'a>(arcs: &'a Vec<DirectArc>) -> Vec<Vec<&'a DirectArc>> {
    let mut res = Vec::new();

    // generate a list of all operators occuring in the list of direct arcs
    let mut indices: Vec<i32> = arcs
        .iter()
        .filter_map(|a| match a.source {
            ArcSource::Local(ref src) => Some(src.operator),
            _ => None,
        })
        .collect();
    indices.sort_unstable();
    indices.dedup();

    // inspect the out arcs of every operator separately, looking for out-arc pairs
    for op in indices {
        let mut relevant_arcs = get_out_arcs(&op, arcs);

        relevant_arcs.sort_unstable_by_key(|arc| match arc.source {
            ArcSource::Local(ref src) => src.index,
            ref x => panic!("Unsupported operator type: {:?}", x),
        });

        let mut selected_arcs = Vec::new();
        let mut current_idx: i32 = 0;

        // group into vec
        for arc in relevant_arcs.drain(..) {
            match arc.source {
                ArcSource::Local(ref src) => {
                    if selected_arcs.len() != 0 {
                        if src.index == current_idx {
                            // we are still filling the same group
                            selected_arcs.push(arc);
                        } else {
                            // start of a new group -> store away the old one
                            if selected_arcs.len() == 1 {
                                selected_arcs.clear();
                            } else {
                                res.push(selected_arcs);
                                selected_arcs = Vec::new();
                            }
                            current_idx = src.index;
                            selected_arcs.push(arc);
                        }
                    } else {
                        selected_arcs.push(arc);
                        current_idx = src.index;
                    }
                }
                ref x => panic!("Unsupported operator type: {:?}", x),
            }
        }
        if selected_arcs.len() > 1 {
            res.push(selected_arcs);
        }
    }

    res
}

/// Generate the variable name for a pair arc
fn generate_pair_arc_var<'a>(arc: &'a DirectArc) -> Ident {
    match arc.source {
        ArcSource::Local(ref src) => Ident::new(
            &format!("sf_{}_out_{}", src.operator, src.index),
            Span::call_site(),
        ),
        ref x => panic!("Unsupported operator type: {:?}", x),
    }
}

pub fn generate_arcs(compiled: &OhuaData) -> TokenStream {
    let mut arcs: Vec<DirectArc> = compiled.graph.arcs.direct.clone();

    arcs.retain(filter_env_arc);

    // separate normal arcs and dead arcs
    let (normal_arcs, dead_ends): (Vec<DirectArc>, Vec<DirectArc>) =
        arcs.clone().into_iter().partition(|arc| {
            compiled
                .graph
                .operators
                .iter()
                .find(|x| x.operatorId == arc.target.operator)
                .is_some()
        });

    let outs = normal_arcs
        .iter()
        .map(|arc| generate_out_arc_var(&arc, &(compiled.graph.operators)));
    let ins = normal_arcs
        .iter()
        .map(|arc| generate_var_for_in_arc(&(arc.target.operator), &(arc.target.index)));

    let dead_outs = dead_ends
        .iter()
        .map(|arc| generate_out_arc_var(&arc, &(compiled.graph.operators)));

    let index_pairs = find_out_port_pairs(&arcs);
    let pair_ins: Vec<Ident> = index_pairs
        .iter()
        .map(|pair| generate_pair_arc_var(pair[0]))
        .collect();
    let pair_args: Vec<Vec<Ident>> = index_pairs
        .iter()
        .map(|pair| {
            pair.iter()
                .map(|it| generate_out_arc_var(it, &(compiled.graph.operators)))
                .collect()
        })
        .collect();

    let state_outs = compiled
        .graph
        .arcs
        .state
        .iter()
        .map(|arc| generate_send_var_for_state_arc(&arc, &(compiled.graph.operators)));
    let state_ins = compiled
        .graph
        .arcs
        .state
        .iter()
        .map(|arc| generate_recv_var_for_state_arc(&(arc.target)));

    quote! {
        #(let (#outs, #ins) = std::sync::mpsc::channel();)*
        #(let #dead_outs = DeadEndArc::default();)*
        #(let (#state_outs, #state_ins) = std::sync::mpsc::channel();)*
        #(let #pair_ins = DispatchQueue::new(vec![#(#pair_args,)*]);)*
    }
}

fn get_call_reference(op_type: &OperatorType) -> Ident {
    // according to the following reference, the name of the function is also an Ident;
    // https://docs.rs/syn/0.15/syn/struct.ExprMethodCall.html
    // but the Ident can not be ns1::f. so how are these calls parsed then?
    // if this happens then we might have to decompose qbName into its name and the namespace.
    Ident::new(&op_type.qbName, Span::call_site())
}

// The `ctrl` is special and I do not know what other "special" operators we might want to build.
// The main point is this: I intended to replace/extend the `loop` construct with something that
// interfaces with a scheduler. Let's see if I can get rid of this and accomplish the same just
// via the arcs.
fn generate_operator_code(op_name: Ident, call_args: Vec<TokenStream>) -> TokenStream {
    let name_str = op_name.to_string();
    if name_str.starts_with("ctrl_") || name_str.starts_with("recur_") {
        quote! { #op_name(#(&#call_args),*)?; Ok(()) }
    } else {
        quote! {
            loop{
                #op_name(#(&#call_args),*)?;
            }
        }
    }
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
            let mut call_args = generate_in_arcs_vec(
                &(op.operatorId),
                &(op.nodeType),
                &(compiled.graph.arcs.direct),
                &Punctuated::new(),
            ); // ops can never have EnvArgs -> invariant broken
            let mut out_arcs = generate_out_arcs_vec(
                &(op.operatorId),
                &(compiled.graph.arcs.direct),
                &(compiled.graph.operators),
            );

            if out_arcs.len() > 0 {
                out_arcs.sort_by_key(|a| match &a.0.source {
                    &Local(ref arc_id) => arc_id.index,
                    other => unimplemented!("sorting by key for {:?}", other),
                });
                let c = out_arcs
                    .iter()
                    .map(|(_, id)| ToTokens::into_token_stream(id));
                call_args.extend(c);
            }

            if op.operatorId == compiled.graph.return_arc.operator {
                // the return_arc is the output port
                call_args.push(quote! { result_snd });
            }

            let op_name = get_call_reference(&op.operatorType);

            if call_args.len() > 0 {
                generate_operator_code(op_name, call_args)
            } else {
                quote! { #op_name() }
            }
        })
        .collect();

    quote! {
        #(tasks.push(Box::new(move || { #op_codes })); )*
    }
}

fn filter_env_arc(arc: &DirectArc) -> bool {
    match arc.source {
        Env(_) => false,
        _ => true,
    }
}

fn generate_sfn_call_code(
    op: &i32,
    sf: Ident,
    call_args: Vec<TokenStream>,
    r: Ident,
    send: TokenStream,
    num_input_arcs: usize,
    state_arcs: &Vec<StateArc>,
) -> TokenStream {
    let is_sfn = match state_arcs.iter().find(|arc| &arc.target == op) {
        Some(_) => true,
        None => false,
    };
    let call_code = quote! {#sf( #(#call_args),* )};
    if is_sfn {
        let state_chan = generate_recv_var_for_state_arc(&op);
        let sfn_code = quote! {
            let #r = state.#call_code;
            #send
        };

        if num_input_arcs > 0 {
            // global state goes along the lines of:
            quote! {
                let state = #state_chan.recv()?;
                loop {
                   #sfn_code
                }
            }
        } else {
            quote! { #sfn_code; Ok(()) }
        }
    } else {
        let sfn_code = quote! {
            let #r = #call_code;
            #send
        };
        if num_input_arcs > 0 {
            quote! {
                loop {
                    #sfn_code
                }
            }
        } else {
            quote! { #sfn_code; Ok(()) }
        }
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
            let mut in_arcs = generate_in_arcs_vec(
                &(op.operatorId),
                &(op.nodeType),
                &(compiled.graph.arcs.direct),
                algo_call_args,
            );
            let orig_in_arcs = get_in_arcs(&(op.operatorId), &(compiled.graph.arcs.direct));
            let zipped_in_arcs: Vec<(&&DirectArc, TokenStream)> =
                orig_in_arcs.iter().zip(in_arcs.drain(..)).collect();

            // FIXME What was that needed for? Passing one env arg to a function more than once?
            // // determine if cloning is necessary and apply it if so
            // let mut seen_env_arcs = HashMap::new();
            // let mut seen_local_arc = false;
            // for pos in 0..zipped_in_arcs.len() {
            //     match zipped_in_arcs[pos].0.source{
            //         Env(ref e) => match e {
            //             EnvRefLit(x) => {
            //                 if let Some(old_pos) = seen_env_arcs.insert(x, pos) {
            //                     // the value is present, clone the old one
            //                     let old_ident = zipped_in_arcs[old_pos].1.clone();
            //                     zipped_in_arcs[old_pos].1 = quote!{ #old_ident.clone() };
            //                 }
            //             },
            //             _ => unimplemented!("generate_sfns -> other literals"),
            //         },
            //         Local(_) => {
            //             seen_local_arc = true;
            //         }
            //     }
            // }
            //
            // // necessary workaround to add cloning for non-"env arc only" operators where they are used in a loop
            // if seen_local_arc {
            //     for (_, index) in seen_env_arcs {
            //         let old_ident = zipped_in_arcs[index].1.clone();
            //         zipped_in_arcs[index].1 = quote!{ #old_ident.clone() };
            //     }
            // }

            // the following assignment is necessary to keep the borrowed value created
            // by the function alive just long enough to wait until the borrowed values
            // are dropped after the unzip
            let mut tmp_out_arcs_vec = generate_out_arcs_vec(
                &op.operatorId,
                &(compiled.graph.arcs.direct),
                &(compiled.graph.operators),
            );
            let out_arcs = tmp_out_arcs_vec
                .drain(..)
                .unzip::<&DirectArc, Ident, Vec<&DirectArc>, Vec<Ident>>()
                .1;

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
                .map(|(orig_arc, code)| match orig_arc.source {
                    Env(_) => code.clone().clone(),
                    Local(_) => quote! { #code.recv()? },
                })
                .collect();

            generate_sfn_call_code(
                &op.operatorId,
                sf,
                call_args,
                r,
                send,
                num_input_arcs,
                &compiled.graph.arcs.state,
            )
        })
        .collect();

    quote! {
        let mut tasks: Vec<Box<dyn FnOnce() -> Result<(), RunError> + Send + 'static>> = Vec::new();
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
                quote! { result_snd.dispatch(#r)?; }
            } else {
                quote! {} // drop
            }
        }
        1 => {
            let o = &outputs[0];
            quote! { #o.dispatch(#r)? }
        }
        _ => {
            let results: Vec<Ident> = outputs.iter().map(|_| r.clone()).collect();
            quote! {
                #(#outputs.dispatch(#results.clone())?);*;
            }
        }
    }
}

fn generate_app_namespaces(operators: &Vec<OperatorType>) -> Vec<TokenStream> {
    let mut namespaces = BTreeSet::new();
    for op in operators {
        // ignore imports in the root
        if op.qbNamespace.is_empty() {
            continue;
        }
        let mut r = op.qbNamespace.to_vec();
        r.push(op.qbName.to_string());

        namespaces.insert(r);
    }

    namespaces
        .iter()
        .map(|r| {
            let initial_val = Ident::new(&r[0], Span::call_site());
            let ns_id = r
                .iter()
                .skip(1) // used as initial state for folding (assertion: must have at least one element!)
                .fold(quote! { #initial_val }, |state, curr| {
                    let n = Ident::new(&curr, Span::call_site());
                    quote! {
                        #state::#n
                    }
                });

            quote! {
                use #ns_id;
            }
        })
        .collect()
}

fn generate_imports(operators: &Vec<Operator>, arcs: &Vec<DirectArc>) -> TokenStream {
    let op_types = operators.iter().map(|op| op.operatorType.clone()).collect();
    let app_namespaces = generate_app_namespaces(&op_types);

    let mut arcs1 = arcs.clone();

    let fn_lit_types = arcs1
        .drain(..)
        .filter(|arc| match arc.clone().source {
            Env(e) => match e {
                FunRefLit { contents: _ } => true,
                _ => false,
            },
            _ => false,
        })
        .map(|arc| match arc.source {
            Env(e) => match e {
                FunRefLit { contents: op } => op,
                _ => panic!("Invariant broken!"),
            },
            _ => panic!("Invariant broken!"),
        })
        .collect();
    let fn_lit_namespaces = generate_app_namespaces(&fn_lit_types);

    quote! {
        use std::sync::mpsc::Receiver;
        use ohua_runtime::*;
        use ohua_runtime::arcs::*;

        use ohua_runtime::lang::{send_once, Unit};

        #(#app_namespaces)*
        #(#fn_lit_namespaces)*
    }
}

// TODO: generalize this
fn generate_ctrls(compiled_algo: &mut OhuaData) -> TokenStream {
    let mut num_args: Vec<usize> = compiled_algo
        .graph
        .operators
        .iter()
        .filter(|op| is_runtime_op(op) && op.operatorType.qbName.as_str() == "ctrl")
        .map(|op| {
            let num_args = get_num_inputs(&op.operatorId, &compiled_algo.graph.arcs.direct);
            num_args - 1
        })
        .collect();
    num_args.sort();
    num_args.dedup();

    let code: Vec<TokenStream> = num_args
        .drain(..)
        .map(|num| generate_ctrl_operator(num))
        .collect();

    let mut ops: Vec<Operator> = compiled_algo.graph.operators.drain(..).collect();

    compiled_algo.graph.operators = ops
        .drain(..)
        .map(|mut op| {
            if is_runtime_op(&op) && op.operatorType.qbName.as_str() == "ctrl" {
                let num_args = get_num_inputs(&op.operatorId, &compiled_algo.graph.arcs.direct);
                op.operatorType.qbNamespace = vec![];
                op.operatorType.qbName = format!("ctrl_{}", num_args - 1);
            }
            op
        })
        .collect();

    quote! {
        #(#code)*
    }
}

fn find_nth_info(op_id: &i32, direct_arcs: &Vec<DirectArc>) -> (i32, i32) {
    let mut in_arcs = get_in_arcs(op_id, direct_arcs);
    assert!(in_arcs.len() == 3);
    in_arcs.sort_by_key(|arc| arc.target.index);
    let num_arc = in_arcs.get(0).expect("Impossible!");
    let len_arc = in_arcs.get(1).expect("Impossible");
    let num = match num_arc.source {
        Env(ref e) => match e {
            NumericLit { content: i } => *i,
            _ => panic!("Compiler invariant broken!"),
        },
        _ => panic!("Compiler invariant broken!"),
    };
    let len = match len_arc.source {
        Env(ref e) => match e {
            NumericLit { content: i } => *i,
            _ => panic!("Compiler invariant broken!"),
        },
        _ => panic!("Compiler invariant broken!"),
    };
    (num, len)
}

fn is_nth(op_id: &i32, ops: &Vec<Operator>) -> bool {
    let op = ops.iter().find(|op| &op.operatorId == op_id);
    match op {
        Some(o) => is_runtime_op(o) && o.operatorType.qbName.as_str() == "nth",
        None => false,
    }
}

fn generate_nths(compiled_algo: &mut OhuaData) -> TokenStream {
    let mut nths: Vec<(i32, i32, i32)> = compiled_algo
        .graph
        .operators
        .iter()
        .filter(|op| is_runtime_op(op) && op.operatorType.qbName.as_str() == "nth")
        .map(|op| {
            let (idx, total) = find_nth_info(&op.operatorId, &compiled_algo.graph.arcs.direct);
            (op.operatorId, idx, total)
        })
        .collect();
    nths.sort_unstable();
    nths.dedup();

    let code: Vec<TokenStream> = nths
        .iter()
        .map(|(op, num, len)| generate_nth(op, num, len))
        .collect();

    let mut direct_arcs: Vec<DirectArc> = compiled_algo.graph.arcs.direct.drain(..).collect();
    compiled_algo.graph.arcs.direct = direct_arcs
        .drain(..)
        .filter(|arc| {
            let nth = is_nth(&arc.target.operator, &compiled_algo.graph.operators);
            if nth {
                arc.target.index >= 2
            } else {
                true
            }
        })
        .map(|mut arc| {
            let nth = is_nth(&arc.target.operator, &compiled_algo.graph.operators);
            if nth {
                if arc.target.index < 2 {
                    // filtered above
                    panic!("Impossible!");
                } else if arc.target.index == 2 {
                    arc.target.index = 0;
                } else {
                    // assertion checked elsewhere already
                    panic!("Impossible!");
                }
            }

            arc
        })
        .collect();

    let mut ops: Vec<Operator> = compiled_algo.graph.operators.drain(..).collect();
    compiled_algo.graph.operators = ops
        .drain(..)
        .map(|mut op| {
            match nths.iter().find(|(id, _, _)| id == &op.operatorId) {
                Some((op_id, idx, total)) => {
                    op.operatorType.qbNamespace = vec![];
                    op.operatorType.qbName = format!("nth_op{}_{}_{}", op_id, idx, total);
                }
                None => (),
            }
            op
        })
        .collect();

    quote! {
        #(#code)*
    }
}

mod generate_recur {

    extern crate bit_set;

    use self::bit_set::BitSet;
    use lang::generate_recur;
    use ohua_types::*;
    use proc_macro2::TokenStream;

    type OpId = i32;
    type Arity = usize;
    type Index = i32;

    const OHUA_NAMESPACE: [&str; 2] = ["ohua_runtime", "lang"];
    const RECUR_NAMESPACE: [&str; 2] = OHUA_NAMESPACE;
    const RECUR_NAME: &str = "recurFun";

    fn is_recur(op: &Operator) -> bool {
        let ty = &op.operatorType;
        ty.qbNamespace == RECUR_NAMESPACE
            && ty.qbName == RECUR_NAME
            && op.nodeType == NodeType::OperatorNode
    }

    fn determine_recursion_arity(op_id: OpId, arcs: &Arcs) -> Arity {
        let mut x: Vec<Index> = arcs
            .direct
            .iter()
            .map(|a| &a.target)
            .filter(|t| t.operator == op_id)
            .map(|t| t.index)
            .collect();
        x.dedup();
        (x.len() - 2) / 2
    }

    struct SimpleTracker(BitSet);

    impl SimpleTracker {
        pub fn new() -> SimpleTracker {
            SimpleTracker(BitSet::new())
        }
        pub fn tick(&mut self, i: usize) {
            self.0.insert(i);
        }
        pub fn ticked(&self) -> self::bit_set::Iter<u32> {
            self.0.iter()
        }
    }

    pub fn generate(algo: &mut OhuaData) -> TokenStream {
        let mut arity_tracker = SimpleTracker::new();
        for op in algo.graph.operators.iter_mut() {
            if is_recur(op) {
                let ref mut ty = &mut op.operatorType;
                ty.qbNamespace = Vec::new();
                let arity = determine_recursion_arity(op.operatorId, &algo.graph.arcs);
                ty.qbName = generate_recur::generate_fun_name(arity);
                arity_tracker.tick(arity as usize);
            }
        }

        let code = arity_tracker.ticked().map(generate_recur::generate);

        quote! {
            #(#code)*
        }
    }
}

/// This function captures environment arguments in an `id` operator. The rationale
/// behind this move is that this seemed -- at the time this was conceived -- the best
/// option for enabling multiple uses of a main argument, which requires cloning.
/// One could of course simply clone every occurence of an env arc, but this might
/// produce more problems, requiring unnecessarily `Clone` implementations where normally
/// no clone would be necessary. So the rationale is to encapsulate these env arcs into
/// an operator and let the mechanisms that are already in place for local arcs take care
/// of determining where clones are necessary.
fn handle_environment_arcs(compiled_algo: &mut OhuaData) {
    let mut env_arc_ids = compiled_algo
        .graph
        .arcs
        .direct
        .iter()
        .filter(|a| {
            if let ArcSource::Env(EnvRefLit { content: _ }) = a.source {
                true
            } else {
                false
            }
        })
        .fold(Vec::new(), |mut acc, x| match x.source {
            ArcSource::Env(EnvRefLit { content: num }) => {
                acc.push(num);
                acc
            }
            _ => unreachable!(),
        });
    env_arc_ids.sort();
    env_arc_ids.dedup();

    // find starting number for next operator
    let new_op_id_base: i32 = compiled_algo
        .graph
        .operators
        .iter()
        .map(|o| o.operatorId)
        .max()
        .unwrap_or(0)
        + 1;
    for i in env_arc_ids {
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
        let mut d_arcs: Vec<DirectArc> = compiled_algo.graph.arcs.direct.drain(..).collect();
        compiled_algo.graph.arcs.direct = d_arcs
            .drain(..)
            .map(|mut arc| {
                if let Env(EnvRefLit { content: env_id }) = arc.source.clone() {
                    if i == env_id {
                        arc.source = ArcSource::Local(ArcIdentifier {
                            operator: new_op_id_base + i,
                            index: 0,
                        })
                    }
                }
                arc
            })
            .collect();

        compiled_algo.graph.arcs.direct.push(DirectArc {
            target: ArcIdentifier {
                operator: new_op_id_base + i,
                index: 0,
            },
            source: ArcSource::Env(EnvRefLit { content: i }),
        });
    }
}

pub fn generate_code(
    compiled_algo: &mut OhuaData,
    algo_call_args: &Punctuated<Expr, Token![,]>,
) -> TokenStream {
    run_backend_optimizations(compiled_algo);

    handle_environment_arcs(compiled_algo);
    let ctrl_code = generate_ctrls(compiled_algo);
    let nth_code = generate_nths(compiled_algo);
    //print!("{:?}", compiled_algo.graph.operators);
    let recur_code = generate_recur::generate(compiled_algo);
    // handle_environment_arcs(compiled_algo);
    let header_code = generate_imports(
        &compiled_algo.graph.operators,
        &compiled_algo.graph.arcs.direct,
    );
    let arc_code = generate_arcs(&compiled_algo);
    let sf_code = generate_sfns(&compiled_algo, algo_call_args);
    let op_code = generate_ops(&compiled_algo);

    // Macro hygiene: I can create a variable here and use it throughout the whole call-site of this
    // macro because quote! has Span:call_site() -> call site = call site of the macro!
    // https://github.com/dtolnay/quote
    // https://docs.rs/proc-macro2/0.4/proc_macro2/struct.Span.html#method.call_site
    quote! {
        {
            #header_code

            #ctrl_code
            #nth_code
            #recur_code

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

    use ohua_types::ArcIdentifier;
    use ohua_types::ArcSource;
    use ohua_types::DFGraph;
    use ohua_types::OhuaData;
    use ohua_types::Operator;
    use ohua_types::OperatorType;

    fn producer_consumer(
        prod: OperatorType,
        prod_t: NodeType,
        con: OperatorType,
        con_t: NodeType,
        out_idx: i32,
    ) -> OhuaData {
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
                    direct: vec![DirectArc {
                        target: ArcIdentifier {
                            operator: 1,
                            index: 0,
                        },
                        source: ArcSource::Local(ArcIdentifier {
                            operator: 0,
                            index: out_idx,
                        }),
                    }],
                    state: vec![],
                    dead: vec![],
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
            0,
        );

        let generated_imports =
            generate_imports(&compiled.graph.operators, &compiled.graph.arcs.direct).to_string();
        // println!(
        //     "\nGenerated code for imports:\n{}\n",
        //     &(generated_imports.replace(";", ";\n"))
        // );
        assert!("use std :: sync :: mpsc :: Receiver ; use ohua_runtime :: * ; use ohua_runtime :: arcs :: * ; use ohua_runtime :: lang :: { send_once , Unit } ; use ns1 :: some_sfn ; use ns2 :: some_other_sfn ;" == generated_imports);

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
        assert!("let mut tasks : Vec < Box < dyn FnOnce ( ) -> Result < ( ) , RunError > + Send + 'static >> = Vec :: new ( ) ; tasks . push ( Box :: new ( move || { let r = some_sfn ( ) ; sf_0_out_0__sf_1_in_0 . dispatch ( r ) ? ; Ok ( ( ) ) } ) ) ; tasks . push ( Box :: new ( move || { loop { let r = some_other_sfn ( sf_1_in_0 . recv ( ) ? ) ; result_snd . dispatch ( r ) ? ; } } ) ) ;" == generated_sfns);
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
        assert!("tasks . push ( Box :: new ( move || { loop { some_op ( & sf_0_out_0__sf_1_in_0 ) ? ; } } ) ) ; tasks . push ( Box :: new ( move || { loop { some_other_op ( & sf_1_in_0 , & result_snd ) ? ; } } ) ) ;" == generated_ops);
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
                        source: ArcSource::Env(EnvRefLit { content: 0 }),
                    }],
                    state: vec![],
                    dead: vec![],
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
        assert!("let mut tasks : Vec < Box < dyn FnOnce ( ) -> Result < ( ) , RunError > + Send + 'static >> = Vec :: new ( ) ; tasks . push ( Box :: new ( move || { let r = some_sfn ( arg1 ) ; ; Ok ( ( ) ) } ) ) ;" == generated_sfns);
    }
}
