use ohua_types::{ArcIdentifier, ArcSource, DirectArc, OhuaData};

/// HACK: Removes dead end arcs and appends them to the direct arc list with
/// target (0, 0). This is due to the fact that at the point when DeadArcs were
/// introduced, DirectArcs were already so deeply embedded into the arc generation,
/// that this seemed simpler.
fn process_dead_ends(compiled_algo: &mut OhuaData) {
    for dead_arc in compiled_algo.graph.arcs.dead.drain(..) {
        compiled_algo.graph.arcs.direct.push(DirectArc {
            target: ArcIdentifier {
                operator: 0,
                index: 0,
            },
            source: ArcSource::Local(dead_arc.source),
        });
    }
}

/// Run a set of backend-specific optimizations on the `OhuaData` structure
pub fn run_backend_optimizations(compiled_algo: &mut OhuaData) {
    process_dead_ends(compiled_algo);
}
