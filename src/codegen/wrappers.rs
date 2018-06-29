//! Wrapper generator for stateful functions.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Result, Write};

use ohua_types::*;
use type_extract::TypeKnowledgeBase;

/// Data Type that describes the type of output an operator produces
pub enum Output {
    /// Only a single output is produced, which is denoted by port `-1` in the DFG dump
    Single(usize),
    /// The produced output is a tuple which is destructured into several ports
    Tuple(Vec<usize>),
}

/// Generates a wrapper for a single stateful function.
///
/// Therefore, the function uses a wrapper template and populates it by adding
/// * the necessary re-boxing (typecasts) for all input values
/// * providing the casted arguments to the underlying function
/// * re-boxing the output values into `GenericType`s and returning them as vector.
///   Here, the function return values are cloned, when necessary.
///
/// Returns a string containing the complete wrapper.
fn generate_sfn_wrapper(name: &str, incoming_arcs: usize, output: Output, op_id: i32) -> String {
    let mut skeleton = String::from(include_str!("templates/snippets/wrapper.in"));
    let unpack_arg = "let arg{n} = Box::from(args.pop().unwrap());\n";

    // set the wrapper function name
    let mut mangled_name = name.replace("::", "_");
    mangled_name += &op_id.to_string();
    skeleton = skeleton.replace("{escaped_name}", mangled_name.as_ref());

    // incoming arguments
    let mut incoming = String::new();
    for i in 0..incoming_arcs {
        incoming.push_str(&unpack_arg.replace("{n}", format!("{}", i).as_ref()));
    }
    skeleton = skeleton.replace("{incoming_args}", &incoming);

    // function call (name and arguments)
    let mut fn_args = String::new();
    for i in 0..incoming_arcs {
        if i > 0 {
            fn_args.push_str(", ")
        }
        fn_args.push_str(format!("*arg{}", i).as_ref());
    }

    skeleton = skeleton
        .replace("{name}", name)
        .replace("{fn_args}", &fn_args);

    // outgoing values (unwrapping from tuple (if necessary) and vec appending)
    let mut outgoing = String::new();

    // do not unpack the tuple if there is only one retval!
    if let Output::Tuple(mut outgoing_arcs) = output {
        for (index, count) in outgoing_arcs.drain(..).enumerate() {
            let boxed_arg = format!("Box::from(Box::new(res.{}))", index);
            let boxed_cloned_arg = format!("Box::from(Box::new(res.{}.clone())), ", index);

            // clone the arguments if necessary
            if count > 1 {
                outgoing.push_str(
                    format!("vec![{cloned}], ", cloned = boxed_cloned_arg.repeat(count)).as_ref(),
                );
            } else {
                outgoing.push_str(format!("vec![{}], ", boxed_arg).as_ref());
            }
        }

        skeleton = skeleton.replace("{outgoing_args}", &outgoing);
    } else if let Output::Single(num_uses) = output {
        // TODO: Remove this hack when issue #1 is resolved
        if num_uses == 0 {
            // return an empty vec when the value is unused
            skeleton = skeleton.replace("{outgoing_args}", "");
        } else {
            // the arguments are cloned when necessary
            if num_uses > 1 {
                outgoing.push_str(
                    format!(
                        "vec![{}]",
                        "Box::from(Box::new(res.clone())), ".repeat(num_uses)
                    ).as_ref(),
                );
            } else {
                outgoing.push_str(format!("vec![{}]", "Box::from(Box::new(res))").as_ref());
            }
        }

        skeleton = skeleton.replace("{outgoing_args}", &outgoing);
    } else {
        // There _is_ no other option!
        unreachable!();
    }

    skeleton
}

/// Function that analyzes the DFG provided by the user. Generates set containing all namespaces.
///
/// The hashset of namespaces is used to generate the correct imports to have all functions in scope.
fn analyze_namespaces(ohuadata: &OhuaData) -> HashSet<String> {
    let mut namespaces = HashSet::new();

    // for each operator in the DFG, check the arcs and count the number of incoming and outgoing arcs
    for op in &ohuadata.graph.operators {
        let namespace = op.operatorType
            .qbNamespace
            .iter()
            .fold(String::new(), |acc, ref x| {
                acc.to_owned() + if !acc.is_empty() { "::" } else { "" } + &x
            });

        if !namespace.starts_with("ohua::lang") {
            namespaces.insert(namespace);
        }
    }

    namespaces
}

/// Retrieves the I/O count for an operator, given its ID
fn get_operator_io_by_id(id: i32, arcs: &[Arc], return_arc: &ArcIdentifier) -> (usize, Output) {
    let mut inputs = 0;
    let mut mult_outputs = Vec::new();
    let mut single_outputs = 0;
    let mut is_single_output = false;

    // check DFG for a matching input or output id
    for arc in arcs {
        // check for possible input match
        if arc.target.operator == id {
            inputs += 1;
        }

        // check for possible output match
        if let ValueType::LocalVal(ref ident) = arc.source.val {
            if ident.operator == id {
                if ident.index == -1 {
                    is_single_output = true;
                    single_outputs += 1;
                } else {
                    let port = ident.index as usize;
                    // when we hit a port that is beyond the output vector's current reach, resize
                    if mult_outputs.len() < (port + 1) {
                        mult_outputs.resize(port + 1, 0);
                    }

                    mult_outputs[port] += 1;
                }
            }
        }
    }

    // input arcs are not inspected as they are part of the DFG (env arcs)

    // inspect return arc
    if return_arc.operator == id {
        if return_arc.index == -1 {
            is_single_output = true;
            single_outputs += 1;
        } else {
            let port = return_arc.index as usize;
            // when we hit a port that is beyond the output vector's current reach, resize
            if mult_outputs.len() < (port + 1) {
                mult_outputs.resize(port + 1, 0);
            }

            mult_outputs[port] += 1;
        }
    }

    // verify that the above yielded a consistent state
    if (is_single_output && !mult_outputs.is_empty()) || (!is_single_output && single_outputs > 0) {
        panic!("Encountered inconsistent Operator that has tuple-destructuring arcs and single-item arcs!");
    }

    let outputs = match is_single_output {
        true => Output::Single(single_outputs),
        false => Output::Tuple(mult_outputs),
    };

    (inputs, outputs)
}

/// Wraps the arguments provided to the algorithm into an operator to allow on-demand
/// cloning and a clean integration into the DFG.
///
/// Also creates the necessary new operators for the mainarg wrappers.
fn generate_mainarg_wrappers(
    first_id: i32,
    ohuadata: &OhuaData,
    mainarg_types: &[String],
) -> (Vec<Operator>, String) {
    let template = include_str!("templates/snippets/mainarg.in");

    let mut operators = Vec::new();
    let mut wrapper_code = String::new();

    for arg_no in 0..ohuadata.mainArity {
        // find out, whether the mainarg has to be cloned
        let num_uses = ohuadata.graph.arcs.iter().fold(0, |acc, ref x| {
            if let ValueType::EnvironmentVal(e) = x.source.val {
                if e == arg_no {
                    return acc + 1;
                }
            }
            acc
        });

        // generate wrapper code
        let mut wrapper = String::from(template).replace("{n}", format!("{}", arg_no).as_str());
        wrapper = wrapper.replace("{arg_type}", mainarg_types[arg_no as usize].as_str());

        // there is an optimization: unused mainargs will not be wrapped!
        match num_uses {
            0 => continue,
            1 => {
                wrapper = wrapper.replace(
                    "{argument}",
                    format!("Box::from(Box::new({}))", "arg").as_str(),
                )
            }
            _ => {
                wrapper = wrapper.replace(
                    "{argument}",
                    format!("Box::from(Box::new({}.clone())), ", "arg")
                        .repeat(num_uses)
                        .as_str(),
                )
            }
        }
        wrapper_code.push_str(wrapper.as_str());

        // generate new operator for respective argument
        let fn_name = format!("mainarg{}", arg_no);
        operators.push(Operator {
            operatorId: first_id + arg_no,
            operatorType: OperatorType {
                qbNamespace: vec![],
                qbName: fn_name.clone(),
                func: fn_name,
                op_type: OpType::default()
            },
        });
    }

    (operators, wrapper_code)
}

/// The main entry point to start wrapper generation.
///
/// This function analyzes the DFG, generates the necessary imports, function wrappers and
/// main argument wrappers. In the process, the `OhuaData` structure is rewritten to add
/// links to the corresponding wrapped functions and add the main argument wrappers.
///
/// Returns either an IO error when opening/writing to the file failed or the updated ohua data structure
pub fn generate_wrappers(
    mut ohuadata: OhuaData,
    lookupinfo: &TypeKnowledgeBase,
    target_file: &str,
) -> Result<OhuaData> {
    // analyze the dataflow graph
    let namespaces = analyze_namespaces(&ohuadata);

    let skeleton = include_str!("templates/wrappers.rs");

    // generate imports
    let mut imports = namespaces
        .iter()
        .fold(String::new(), |acc, ref x| acc + "use " + x + ";\n");

    // generate function wrappers
    let mut func_wrapper = String::new();
    for op in &mut ohuadata.graph.operators {
        // we would like to handle operators differently, so don't handle them here
        if op.operatorType.qbNamespace != vec!["ohua".to_string(), "lang".to_string()] {
            let (inputs, outputs) = get_operator_io_by_id(
                op.operatorId,
                &ohuadata.graph.arcs,
                &ohuadata.graph.return_arc,
            );

            let fn_name = String::from(op.operatorType.qbNamespace.last().unwrap().as_str()) + "::"
                + op.operatorType.qbName.as_str();

            func_wrapper
                .push_str(generate_sfn_wrapper(&fn_name, inputs, outputs, op.operatorId).as_str());

            op.operatorType.func = fn_name.replace("::", "_") + &op.operatorId.to_string();
        } else {
            // when we leap an operator, mark it and place a failsafe in the `func` field
            op.operatorType.op_type = OpType::OhuaOperator("".into());
            op.operatorType.func = String::from("|_| panic!(\"Undefined operator function!\")");
        }
    }

    // wrap the main arguments
    let first_mainarg = (ohuadata.graph.operators.len() + 1) as i32;
    let (mut mainarg_ops, wrapper_code) =
        generate_mainarg_wrappers(first_mainarg, &ohuadata, &lookupinfo.algo_io.argument_types);

    // register each mainarg operator as target for an input arc
    ohuadata.graph.input_targets.reserve(mainarg_ops.len());
    for op in &mainarg_ops {
        ohuadata.graph.input_targets.push(ArcIdentifier {
            operator: op.operatorId,
            index: 0,
        })
    }

    ohuadata.graph.operators.append(&mut mainarg_ops);

    // after the mainargs have been appended, add the imports for their argument types
    imports += lookupinfo
        .find_imports_for_algo_io()
        .iter()
        .fold(String::from("\n"), |acc, ref x| acc + "use " + x + ";\n")
        .as_str();

    // rewrite the env arcs for any main arguments encountered
    for mut arc in &mut ohuadata.graph.arcs {
        if let ValueType::EnvironmentVal(offset) = arc.source.val {
            arc.source = ArcSource {
                s_type: String::from("local"),
                val: ValueType::LocalVal(ArcIdentifier {
                    operator: first_mainarg + offset,
                    index: -1,
                }),
            };
        }
    }

    File::create(target_file)?.write_fmt(format_args!(
        "{skel}{imp}{func}{args}",
        skel = skeleton,
        imp = imports,
        func = func_wrapper,
        args = wrapper_code
    ))?;

    Ok(ohuadata)
}
