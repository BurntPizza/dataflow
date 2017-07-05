
use pg::Incoming;
use pg::prelude::NodeIndex;
use pg::{graph, stable_graph};

use ::*;

macro_rules! graph_impl {
    ($($t:tt),*; $ty:ty, $neighbors:ty, $iter:ty) => {
        impl<'a, $($t),*> Graph<NodeIndex> for &'a $ty {
            type Iter = $iter;
            type Neighbors = $neighbors;
 
            fn nodes(&self) -> Self::Iter {
                self.node_indices()
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

graph_impl!(N, E; graph::DiGraph<N, E>, graph::Neighbors<'a, E>, graph::NodeIndices);
graph_impl!(N, E; stable_graph::StableDiGraph<N, E>, stable_graph::Neighbors<'a, E>, stable_graph::NodeIndices<'a, N>);
