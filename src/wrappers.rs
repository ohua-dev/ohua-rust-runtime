//! Wrapper generator for stateful functions.

use std::collections::{HashMap, HashSet};
use std::io::{Result, Write};
use std::fs::File;

use types::*;

/// Generates a wrapper for a single stateful function.
///
/// Therefore, the function uses a wrapper template and populates it by adding
/// * the necessary re-boxing (typecasts) for all input values
/// * providing the casted arguments to the underlying function
/// * re-boxing the output values into `GenericType`s and returning them as vector.
///   Here, the function return values are cloned, when necessary.
///
/// Returns a string containing the complete wrapper.
fn wrap_function(name: &str, incoming_arcs: usize, mut outgoing_arcs: Vec<usize>) -> String {
    let mut skeleton = String::from(include_str!("templates/snippets/wrapper.in"));
    let unpack_arg = "let arg{n} = Box::from(args.pop().unwrap());\n";

    // set the wrapper function name
    skeleton = skeleton.replace("{escaped_name}", name.replace("::", "_").as_ref());

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
    if outgoing_arcs.len() > 1 {
        for (index, count) in outgoing_arcs.drain(..).enumerate() {
            let boxed_arg = format!("Box::from(Box::new(res.{}))", index);
            let boxed_cloned_arg = format!("Box::from(Box::new(res.{}.clone()))", index);

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
    } else {
        // TODO: Remove this hack when issue #1 is resolved
        if outgoing_arcs.len() == 0 {
            // return an empty vec when the value is unused
            skeleton = skeleton.replace("{outgoing_args}", "");
        } else {
            let count = outgoing_arcs[0];
            // the arguments are cloned when necessary
            if count > 1 {
                outgoing.push_str(
                    format!(
                        "vec![{}]",
                        "Box::from(Box::new(res.clone())), ".repeat(count)
                    ).as_ref(),
                );
            } else {
                outgoing.push_str(
                    format!("vec![{}]", "Box::from(Box::new(res))".repeat(count)).as_ref(),
                );
            }
        }

        skeleton = skeleton.replace("{outgoing_args}", &outgoing);
    }

    skeleton
}

/// Function that analyzes the DFG provided by the user. Generates a function map
/// and a set containing all namespaces.
///
/// The function map describes, how many incoming and outgoing Arcs an operator has, what
/// function it belongs to and what the corresponding operator number is.
/// This information is used to be able to generate correct wrapper code that retrieves all
/// arguments for a function and provides the correct number of output items (it also allows
/// to clone returned values as necessary before boxing them).
///
/// The hashset of namespaces is used to generate the correct imports to have all functions in scope.
fn analyze_dfg(
    ohuadata: &OhuaData,
) -> (HashMap<String, (usize, Vec<usize>, i32)>, HashSet<String>) {
    let mut function_map: HashMap<String, (usize, Vec<usize>, i32)> = HashMap::new();
    let mut namespaces = HashSet::new();

    // for each operator in the DFG, check the arcs and count the number of incoming and outgoing arcs
    for op in &ohuadata.graph.operators {
        let mut in_count = 0;
        // this vector describes the outgoing arcs, where the first entry is the position and
        // the second the number of arcs originating from that position
        let mut out_count: Vec<(i32, usize)> = Vec::new();

        // check which arcs target the operator and which Arcs describe this Operator as their local source
        for arc in &ohuadata.graph.arcs {
            if arc.target.operator == op.operatorId {
                in_count += 1;
            }
            match &arc.source.val {
                &ValueType::EnvironmentVal(_) => (),
                &ValueType::LocalVal(ref src) => if op.operatorId == src.operator {
                    // perform a sorted insertion into the vector that describes all outgoing arcs
                    let index = if src.index >= 0 {
                        src.index
                    } else {
                        0
                    };

                    match out_count.binary_search_by_key(&index, |x| x.0) {
                        Ok(pos) => out_count[pos].1 += 1,
                        Err(pos) => out_count.insert(pos, (index, 1)),
                    }
                },
            }
        }

        // check the special return value field to account for the Arc that will move the computed result out of the DFG
        if ohuadata.graph.return_arc.operator == op.operatorId {
            // basically do the same as above, insert the information about the outgoing arc into the data structure
            let index = if ohuadata.graph.return_arc.index >= 0 {
                ohuadata.graph.return_arc.index
            } else {
                0
            };

            match out_count.binary_search_by_key(&index, |x| x.0) {
                Ok(pos) => out_count[pos].1 += 1,
                Err(pos) => out_count.insert(pos, (index, 1)),
            }
        }

        let namespace = op.operatorType
            .qbNamespace
            .iter()
            .fold(String::new(), |acc, ref x| {
                acc.to_owned() + if acc.len() > 0 { "::" } else { "" } + &x
            });

        namespaces.insert(namespace);
        let fn_name = String::from(op.operatorType.qbNamespace.last().unwrap().as_str()) + "::"
            + op.operatorType.qbName.as_str();
        // transform the vector, get rid of the first tuple entry
        let out = out_count
            .drain(..)
            .unzip::<i32, usize, Vec<i32>, Vec<usize>>()
            .1;
        function_map.insert(fn_name, (in_count, out, op.operatorId));
    }

    (function_map, namespaces)
}

/// Wraps the arguments provided to the algorithm into an operator to allow on-demand
/// cloning and a clean integration into the DFG.
///
/// Also creates the necessary new operators for the mainarg wrappers.
fn generate_mainarg_wrappers(first_id: i32, ohuadata: &OhuaData, mainarg_types: &Vec<String>) -> (Vec<Operator>, String) {
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
                    format!("Box::from(Box::new({}))", "arg").as_str())
            }
            _ => {
                wrapper = wrapper.replace(
                    "{argument}",
                    format!("Box::from(Box::new({}.clone())), ", "arg").repeat(num_uses).as_str())
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
pub fn generate_wrappers(mut ohuadata: OhuaData, mainarg_types: &Vec<String>, target_file: &str) -> Result<OhuaData> {
    // analyze the dataflow graph
    let (function_map, namespaces) = analyze_dfg(&ohuadata);

    let skeleton = include_str!("templates/wrappers.rs");

    // generate imports
    let imports = namespaces
        .iter()
        .fold(String::new(), |acc, ref x| acc + "use " + x + ";\n");

    // generate function wrappers
    let mut func_wrapper = String::new();
    for (name, io) in function_map {
        func_wrapper.push_str(wrap_function(name.as_str(), io.0, io.1).as_str());

        // link the function to the corresponding operator in the ohua data structure
        if let Ok(pos) = ohuadata
            .graph
            .operators
            .binary_search_by_key(&io.2, |op| op.operatorId)
        {
            ohuadata.graph.operators[pos].operatorType.func = name.replace("::", "_");
        }
    }

    // wrap the main arguments
    let first_mainarg = (ohuadata.graph.operators.len() + 1) as i32;
    let (mut mainarg_ops, wrapper_code) = generate_mainarg_wrappers(first_mainarg, &ohuadata, mainarg_types);

    // register each mainarg operator as target for an input arc
    ohuadata.graph.input_targets.reserve(mainarg_ops.len());
    for op in &mainarg_ops {
        ohuadata.graph.input_targets.push(ArcIdentifier {operator: op.operatorId, index: 0})
    }

    ohuadata.graph.operators.append(&mut mainarg_ops);

    // rewrite the env arcs for any main arguments encountered
    for mut arc in ohuadata.graph.arcs.iter_mut() {
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
