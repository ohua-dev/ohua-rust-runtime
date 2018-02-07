use std::collections::{HashMap, HashSet};
use std::io::{Result, Write};
use std::fs::File;

use types::*;

pub fn wrap_function(name: &str, incoming_arcs: usize, mut outgoing_arcs: Vec<usize>) -> String {
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

    println!("Wrapping {}: {:?}", name, outgoing_arcs);

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
            skeleton = skeleton.replace("{outgoing_args}", "vec![Box::from(Box::new(res))]");
        } else {
            let count = outgoing_arcs[0];
            // the arguments are cloned when necessary
            if count > 0 {
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

    // if outgoing_arcs.len() > 1 {
    //     for i in 0..outgoing_arcs {
    //         if i > 0 {
    //         outgoing.push_str(", ")
    //         }
    //         outgoing.push_str(format!("Box::from(Box::new(res.{}))", i).as_ref());
    //     }
    // } else {
    //
    // }

    skeleton
}

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
                        Ok(pos) => {
                            println!("Incrementing pos {} -- was {}.", pos, out_count[pos].1);
                            out_count[pos].1 += 1
                        }
                        Err(pos) => {
                            println!("Inserting 1 at {}", pos);
                            out_count.insert(pos, (index, 1))
                        }
                    }
                },
            }
            println!("{:?}", out_count);
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

fn get_argument(arg_no: i32) -> String {
    match arg_no {
        0 => String::from("8"),
        _ => String::new(),
    }
}

fn generate_mainarg_wrappers(first_id: i32, ohuadata: &OhuaData) -> (Vec<Operator>, String) {
    let template = include_str!("templates/snippets/mainarg.in");

    let mut operators = Vec::new();
    let mut wrapper_code = String::new();

    for arg_no in 0..ohuadata.mainArity {
        // generate wrapper code
        let wrapper = String::from(template)
            .replace("{n}", format!("{}", arg_no).as_str())
            .replace("{argument}", get_argument(arg_no).as_str());
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

pub fn generate_wrappers(mut ohuadata: OhuaData, target_file: &str) -> Result<OhuaData> {
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

        // link the function to the ohua data structure
        if let Ok(pos) = ohuadata
            .graph
            .operators
            .binary_search_by_key(&io.2, |ref op| op.operatorId)
        {
            ohuadata.graph.operators[pos].operatorType.func = name.replace("::", "_");
        }
    }

    let first_mainarg = (ohuadata.graph.operators.len() + 1) as i32;
    let (mut mainarg_ops, arg_wrappers) = generate_mainarg_wrappers(first_mainarg, &ohuadata);
    ohuadata.graph.operators.append(&mut mainarg_ops);

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
        args = arg_wrappers
    ))?;

    Ok(ohuadata)
}
