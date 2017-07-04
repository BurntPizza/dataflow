
use pg::Incoming;
use pg::prelude::NodeIndex;
use pg::{graph, stable_graph};

use ::*;

macro_rules! graph_impl {
    ($($t:tt),*; $ty:ty, $neighbors:ty) => {
        impl<'a, $($t),*> Graph<NodeIndex> for &'a $ty {
            type Neighbors = $neighbors;

            fn entry(&self) -> NodeIndex {
                self.node_indices()
                    .filter(|i| {
                        self.neighbors_directed(*i, Incoming).count() == 0
                    })
                    .next()
                    .unwrap()
            }

            fn exit(&self) -> NodeIndex {
                self.node_indices()
                    .rev()
                    .filter(|i| self.neighbors(*i).count() == 0)
                    .next()
                    .unwrap()
            }

            fn immediate_predecessors(&self, node: NodeIndex) -> Self::Neighbors {
                self.neighbors_directed(node, Incoming)
            }

            fn immediate_successors(&self, node: NodeIndex) -> Self::Neighbors {
                self.neighbors(node)
            }
        }
    };
}

graph_impl!(N, E; graph::DiGraph<N, E>, graph::Neighbors<'a, E>);
graph_impl!(N, E; stable_graph::StableDiGraph<N, E>, stable_graph::Neighbors<'a, E>);
