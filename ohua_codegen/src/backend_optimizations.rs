use ohua_types::{ArcIdentifier, ArcSource, DirectArc, OhuaData};
use typedgen::get_out_arcs;

/// After severe issues with `ctrl` operators that have empty argument lists,
/// this optimization pass removes any such operators, leaving behind (implicit)
/// DeadEndArcs
fn remove_unused_ctrl(compiled_algo: &mut OhuaData) {
    // need to pull the ops from the struct first due to borrowing
    let mut ops = vec![];
    std::mem::swap(&mut compiled_algo.graph.operators, &mut ops);

    // drop operators that are `ctrl` and have no inputs
    ops.retain(|op| {
        !(op.operatorType.qbNamespace == vec!["ohua_runtime", "lang"]
            && op.operatorType.qbName.as_str() == "ctrl"
            && get_out_arcs(&op.operatorId, &compiled_algo.graph.arcs.direct).is_empty())
    });

    compiled_algo.graph.operators = ops;
}

fn resolve_missing_data_out_ports(compiled_algo: &mut OhuaData) {
    // find all smapFun operators
    let op_nums: Vec<i32> = compiled_algo
        .graph
        .operators
        .iter()
        .filter(|op| {
            op.operatorType.qbNamespace == vec!["ohua_runtime", "lang"]
                && op.operatorType.qbName.as_str() == "smapFun"
        })
        .map(|op| op.operatorId)
        .collect();

    // check, if a smapFun operator has been optimized and the `data_out` arc was removed
    for op_id in op_nums {
        if compiled_algo
            .graph
            .arcs
            .direct
            .iter()
            .find(|arc| match arc.source {
                ArcSource::Local(ArcIdentifier {
                    operator: id,
                    index: 0,
                }) => id == op_id,
                _ => false,
            })
            .is_none()
        {
            // if so, add a DeadEndArc
            compiled_algo.graph.arcs.direct.push(DirectArc {
                target: ArcIdentifier {
                    operator: 0,
                    index: 0,
                },
                source: ArcSource::Local(ArcIdentifier {
                    operator: op_id,
                    index: 0,
                }),
            });
        }
    }
}

/// Run a set of backend-specific optimizations on the `OhuaData` structure
pub fn run_backend_optimizations(compiled_algo: &mut OhuaData) {
    remove_unused_ctrl(compiled_algo);
    resolve_missing_data_out_ports(compiled_algo);
}
