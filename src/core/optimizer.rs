use petgraph::stable_graph::NodeIndex;

use crate::graph::Graph;

pub trait GraphOptimizer {
    /// Run an optimization pass
    fn optimize(&self, graph: &mut Graph);
}

impl GraphOptimizer for () {
    fn optimize(&self, _: &mut Graph) {}
}

macro_rules! tuple_impls {
    ([$($name:ident),+] , [$($idx:tt),+]) => {
        impl<
        $($name:
            GraphOptimizer, )+
        > GraphOptimizer for ($($name,)+) {
            fn optimize(&self, graph: &mut Graph) {
                $(self.$idx.optimize(graph);)+
            }
        }
    };
}

tuple_impls!([M1], [0]);
tuple_impls!([M1, M2], [0, 1]);
tuple_impls!([M1, M2, M3], [0, 1, 2]);
tuple_impls!([M1, M2, M3, M4], [0, 1, 2, 3]);
tuple_impls!([M1, M2, M3, M4, M5], [0, 1, 2, 3, 4]);
tuple_impls!([M1, M2, M3, M4, M5, M6], [0, 1, 2, 3, 4, 5]);
tuple_impls!([M1, M2, M3, M4, M5, M6, M7], [0, 1, 2, 3, 4, 5, 6]);
tuple_impls!([M1, M2, M3, M4, M5, M6, M7, M8], [0, 1, 2, 3, 4, 5, 6, 7]);
tuple_impls!(
    [M1, M2, M3, M4, M5, M6, M7, M8, M9],
    [0, 1, 2, 3, 4, 5, 6, 7, 8]
);
tuple_impls!(
    [M1, M2, M3, M4, M5, M6, M7, M8, M9, M10],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
);

// Graph Selector
pub trait GraphSelector {
    fn is_valid(&self, node: NodeIndex, graph: &Graph);
}

pub trait GraphSelectorInterface {
    fn or<S: GraphSelector + 'static>(self, other: S) -> OrSelector;
}

impl<T: GraphSelector + 'static> GraphSelectorInterface for T {
    fn or<S: GraphSelector + 'static>(self, other: S) -> OrSelector {
        OrSelector {
            or: vec![Box::new(self), Box::new(other)],
        }
    }
}

pub struct PutSelector<T, S: GraphSelector> {
    reference: *mut T,
    selector: S,
}

pub struct OrSelector {
    or: Vec<Box<dyn GraphSelector>>,
}
