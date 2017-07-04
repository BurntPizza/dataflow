
#[cfg(feature = "petgraph")]
extern crate petgraph as pg;

#[cfg(feature = "petgraph")]
mod petgraph;

#[cfg(feature = "petgraph")]
pub use petgraph::*;

pub use Direction::*;
pub use Mode::*;

use std::hash::Hash;
use std::collections::VecDeque;

pub trait Graph<Idx> {
    type Neighbors: IntoIterator<Item = Idx>;

    fn entry(&self) -> Idx;
    fn exit(&self) -> Idx;
    fn immediate_predecessors(&self, node: Idx) -> Self::Neighbors;
    fn immediate_successors(&self, node: Idx) -> Self::Neighbors;
}

pub trait State {
    type NodeIdx;
    type Idx;
    type Set: Set<Self::Idx>;

    fn gen(&self, i: Self::NodeIdx) -> &Self::Set;
    fn kill(&self, i: Self::NodeIdx) -> &Self::Set;
    fn in_facts(&mut self, i: Self::NodeIdx) -> &mut Self::Set;
    fn out_facts(&mut self, i: Self::NodeIdx) -> &mut Self::Set;
}

pub trait Set<T>: Clone + PartialEq {
    fn empty() -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn intersection(self, other: &Self) -> Self;
    fn union(self, other: &Self) -> Self;
}

pub trait Analysis {
    type State: State;
    fn from(state: Self::State) -> Self;
}

pub enum Direction {
    Forward,
    Backward,
}

pub enum Mode {
    May,
    Must,
}

pub fn analyze<A, G, S, State, Idx, NodeIdx>(
    g: G,
    initial: State,
    dir: Direction,
    mode: Mode,
) -> A
where
    A: Analysis<State = State>,
    G: Graph<NodeIdx>,
    S: Set<Idx>,
    NodeIdx: Copy,
    Idx: Eq + Hash + Copy,
    State: self::State<Idx = Idx, NodeIdx = NodeIdx, Set = S>,
{
    let mut state = initial;

    let op = match mode {
        May => S::union,
        Must => S::intersection,
    };

    let start = match dir {
        Forward => g.entry(),
        Backward => g.exit(),
    };

    let inputs = |n| match dir {
        Forward => g.immediate_predecessors(n),
        Backward => g.immediate_successors(n),
    };

    let mut work_list = VecDeque::from(vec![start]);

    while let Some(n) = work_list.pop_front() {
        let a = {
            let mut a = S::empty();
            for i in inputs(n) {
                let pm = match dir {
                    Forward => state.out_facts(i),
                    Backward => state.in_facts(i),
                };
                a = op(a, pm);
            }
            a
        };

        let b = a.difference(state.kill(n)).union(state.gen(n));

        let (state_b, next) = match dir {
            Forward => {
                *state.in_facts(n) = a;
                (state.out_facts(n), g.immediate_successors(n))
            }
            Backward => {
                *state.out_facts(n) = a;
                (state.in_facts(n), g.immediate_predecessors(n))
            }
        };

        if *state_b != b {
            *state_b = b;
            work_list.extend(next);
        }
    }

    A::from(state)
}

use std::collections::HashSet;

impl<T: Eq + Hash + Clone> Set<T> for HashSet<T> {
    fn empty() -> Self {
        HashSet::new()
    }

    fn difference(&self, other: &Self) -> Self {
        HashSet::difference(self, other).cloned().collect()
    }

    fn union(mut self, other: &Self) -> Self {
        self.extend(other.iter().cloned());
        self
    }

    fn intersection(self, other: &Self) -> Self {
        HashSet::intersection(&self, other).cloned().collect()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
