
use ::*;

impl<'a, N, E> Graph<pg::prelude::NodeIndex> for &'a pg::stable_graph::StableDiGraph<N, E> {
    type Neighbors = pg::stable_graph::Neighbors<'a, E>;

    fn entry(&self) -> pg::prelude::NodeIndex {
        self.node_indices()
            .filter(|i| {
                self.neighbors_directed(*i, pg::Direction::Incoming).count() == 0
            })
            .next()
            .unwrap()
    }

    fn exit(&self) -> pg::prelude::NodeIndex {
        self.node_indices()
            .rev()
            .filter(|i| self.neighbors(*i).count() == 0)
            .next()
            .unwrap()
    }

    fn immediate_predecessors(&self, node: pg::prelude::NodeIndex) -> Self::Neighbors {
        self.neighbors_directed(node, pg::Incoming)
    }

    fn immediate_successors(&self, node: pg::prelude::NodeIndex) -> Self::Neighbors {
        self.neighbors(node)
    }
}
