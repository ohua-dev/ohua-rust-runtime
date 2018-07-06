use std::fs::{File, read_to_string};
use std::io::{Result, Write};
use std::collections::HashSet;

use ohua_types::{OhuaData, OpType, ValueType};
use type_extract::TypeKnowledgeBase;

pub fn gen_tuple_operator(
    data: &mut OhuaData,
    index: usize,
    ty_info: &TypeKnowledgeBase,
    base_path: &str,
) -> Result<()> {
    let mut tuple_file = String::from(include_str!("templates/tuple.rs"));
    let mut tuple_template = String::from(include_str!("templates/snippets/tuple.in"));

    // operators are numbered starting with 1 ¯\_(ツ)_/¯
    let op_id = (index + 1) as i32;

    // replace all placeholders from the template
    tuple_template = tuple_template.replace("{id}", &op_id.to_string());

    let mut arg_unwraps = String::new();
    let mut imports = HashSet::new();
    let mut arg_no = 0;

    /* The lookup of the argument types needed for unwrap is rather tricky,
     * unfortunately. We iterate over all Arcs and backtrack the ones serving
     * as input to our operator. Using the previous node in the DFG (which will
     * alwatys exist because the previous node is either an operator or a trans-
     * formed input arc) we look up the Arc type using the TypeKnowledgeBase. */
    for arc in &data.graph.arcs {
        if arc.target.operator == op_id {
            let (src_id, port) = match arc.source.val {
                ValueType::LocalVal(ref op) => (op.operator as usize, op.index),
                ValueType::EnvironmentVal(_) => continue,
            };

            // retrieve name and namespace of predecessor operator
            let name = &data.graph.operators[src_id - 1].operatorType.qbName;
            let namespace = &data.graph.operators[src_id - 1].operatorType.qbNamespace;

            // use lookup info to extract type info
            if let Some(info) = ty_info.info_for_function(name, namespace) {
                if port >= 0 {
                    let ty_components = match info.return_val.components {
                        Some(ref comp) => comp,
                        None => panic!("Encountered malformed type lookup info. States that it has no components but port # is {}", port),
                    };

                    arg_unwraps += &format!(
                        "let arg{n}: {ty} = *Box::from(op.input[{n}].recv().unwrap());\n    ",
                        n = arg_no,
                        ty = ty_components[port as usize].name
                    );

                    if let Some(ref paths) = ty_components[port as usize].path {
                        for import in paths {
                            imports.insert(import);
                        }
                    }
                } else {
                    arg_unwraps += &format!(
                        "let arg{n}: {ty} = *Box::from(op.input[{n}].recv().unwrap());\n    ",
                        n = arg_no,
                        ty = info.return_val.name
                    );

                    if let Some(ref paths) = info.return_val.path {
                        for import in paths {
                            imports.insert(import);
                        }
                    }
                }

                arg_no += 1;
            }
        }
    }

    // finish template
    tuple_template = tuple_template.replace("{arg_unwrap}", &arg_unwraps);

    let mut args_to_wrap = String::new();
    for i in 0..arg_no {
        args_to_wrap += &format!("arg{n}, ", n = i);
    }
    tuple_template = tuple_template.replace("{args}", &args_to_wrap);

    // link into OhuaData
    data.graph.operators[index].operatorType.op_type =
        OpType::OhuaOperator(format!("operators::tuple::tuple{n}", n = op_id));

    let mod_path = base_path.to_string() + "/tuple.rs";
    if let Ok(mut file_content) = read_to_string(&mod_path) {
        // the file already exists and contains another function
        for import in imports {
            let use_stmt = format!("use {};\n", import);
            // only add use statement if it is not there yet
            if file_content.find(&use_stmt).is_none() {
                file_content.insert_str(0, &use_stmt);
            }
        }

        file_content += "\n\n";
        file_content += tuple_template.as_str();

        // write back to the operator file, overwriting old contents
        File::create(&mod_path)?.write(&file_content.into_bytes())?;
    } else {
        // the file has to be created
        // append imports and function
        tuple_file += imports.drain().fold(String::new(), |acc, imp| acc + &format!("use {};\n", imp)).as_str();
        tuple_file += "\n\n";
        tuple_file += tuple_template.as_str();

        // write everything to the operator file
        File::create(&mod_path)?.write(&tuple_file.into_bytes())?;
    }

    Ok(())
}
