use ohua_types::OhuaData;
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

/// Run a set of backend-specific optimizations on the `OhuaData` structure
pub fn run_backend_optimizations(compiled_algo: &mut OhuaData) {
    remove_unused_ctrl(compiled_algo);
}
