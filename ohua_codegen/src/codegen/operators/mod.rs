use std::collections::HashSet;
use std::fs::{DirBuilder, File};
use std::io::{Result, Write};

mod tuple;

use ohua_types::OhuaData;
use type_extract::TypeKnowledgeBase;

pub fn generate_operators(
    data: &mut OhuaData,
    ty_info: &TypeKnowledgeBase,
    output_base: String,
) -> Result<()> {
    // first of, generate the `operators` submodule
    let output = output_base + "/operators";
    DirBuilder::new().create(output.as_str())?;

    let mut used_operators: HashSet<&str> = HashSet::new();

    // iterate over all operators, find the ohua operators
    for op_id in 0..data.graph.operators.len() {
        if data.graph.operators[op_id].operatorType.qbNamespace
            == vec!["ohua".to_string(), "lang".to_string()]
        {
            // found ohua operator
            // TODO: do a real matching, this is just for debugging
            if data.graph.operators[op_id].operatorType.qbName == "(,)".to_string() {
                tuple::gen_tuple_operator(data, op_id, ty_info, &output)?;
                used_operators.insert("tuple");
            }
        }
    }

    // generate imports from all used operators
    let imports = used_operators
        .drain()
        .fold(String::new(), |acc, imp| acc + &format!("pub mod {};\n", imp));

    // generate the `mod.rs` file
    let modrs = include_str!("templates/mod.rs");
    File::create(output.clone() + "/mod.rs")?.write_fmt(format_args!(
        "{imp}\n{skel}",
        imp = imports,
        skel = modrs
    ))?;

    Ok(())
}

// for op in &mut ohuadata.graph.operators {
//         let (inputs, outputs) = get_operator_io_by_id(
//             op.operatorId,
//             &ohuadata.graph.arcs,
//             &ohuadata.graph.return_arc,
//         );

//         // we would like to handle operators differently, so don't handle them here
//         if op.operatorType.qbNamespace != vec!["ohua".to_string(), "lang".to_string()] {
//             if let Some(wrapper) = wrap_operator(&mut op, inputs, outputs) {
//                 func_wrapper.push_str(wrapper.as_str());
//             }
//         } else {
//             let fn_name = String::from(op.operatorType.qbNamespace.last().unwrap().as_str()) + "::"
//                 + op.operatorType.qbName.as_str();

//             func_wrapper.push_str(generate_sfn_wrapper(&fn_name, inputs, outputs, op.operatorId).as_str());

//             op.operatorType.func = fn_name.replace("::", "_") + &op.operatorId.to_string();
//         }
//     }
