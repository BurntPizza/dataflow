
#[cfg(feature = "petgraph")]
extern crate petgraph as pg;

#[cfg(feature = "petgraph")]
mod petgraph;

#[cfg(feature = "petgraph")]
pub use petgraph::*;

use std::hash::Hash;
use std::collections::VecDeque;

pub trait Graph<Idx> {
    type Iter: IntoIterator<Item = Idx>;
    type Neighbors: IntoIterator<Item = Idx>;

    fn nodes(&self) -> Self::Iter;
    fn immediate_predecessors(&self, node: Idx) -> Self::Neighbors;
    fn immediate_successors(&self, node: Idx) -> Self::Neighbors;
}

pub trait State {
    type NodeIdx: Copy;
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

pub trait Direction {
    #[doc(hidden)]
    fn start<Idx, G: Graph<Idx>>(g: &G) -> VecDeque<Idx>;
    #[doc(hidden)]
    fn inputs<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors;
    #[doc(hidden)]
    fn assign_a<S: State>(state: &mut S, n: S::NodeIdx, a: S::Set);
    #[doc(hidden)]
    fn get_mut_b<S: State>(state: &mut S, n: S::NodeIdx) -> &mut S::Set;
    #[doc(hidden)]
    fn affected<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors;
}

pub struct Forward;
pub struct Backward;

impl Direction for Forward {
    #[doc(hidden)]
    fn start<Idx, G: Graph<Idx>>(g: &G) -> VecDeque<Idx> {
        g.nodes().into_iter().collect()
    }

    #[doc(hidden)]
    fn inputs<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors {
        g.immediate_predecessors(n)
    }

    #[doc(hidden)]
    fn assign_a<S: State>(state: &mut S, n: S::NodeIdx, a: S::Set) {
        *state.in_facts(n) = a;
    }

    #[doc(hidden)]
    fn get_mut_b<S: State>(state: &mut S, n: S::NodeIdx) -> &mut S::Set {
        state.out_facts(n)
    }

    #[doc(hidden)]
    fn affected<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors {
        g.immediate_successors(n)
    }
}

impl Direction for Backward {
    #[doc(hidden)]
    fn start<Idx, G: Graph<Idx>>(g: &G) -> VecDeque<Idx> {
        let v: Vec<_> = g.nodes().into_iter().collect();
        v.into_iter().rev().collect()
    }

    #[doc(hidden)]
    fn inputs<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors {
        g.immediate_successors(n)
    }

    #[doc(hidden)]
    fn assign_a<S: State>(state: &mut S, n: S::NodeIdx, a: S::Set) {
        *state.out_facts(n) = a;
    }

    #[doc(hidden)]
    fn get_mut_b<S: State>(state: &mut S, n: S::NodeIdx) -> &mut S::Set {
        state.in_facts(n)
    }

    #[doc(hidden)]
    fn affected<Idx, G: Graph<Idx>>(g: &G, n: Idx) -> G::Neighbors {
        g.immediate_predecessors(n)
    }
}

pub trait Mode {
    #[doc(hidden)]
    fn op<T, S: Set<T>>(this: S, other: &S) -> S;
}

pub struct May;
pub struct Must;

impl Mode for May {
    #[doc(hidden)]
    fn op<T, S: Set<T>>(this: S, other: &S) -> S {
        this.union(other)
    }
}

impl Mode for Must {
    #[doc(hidden)]
    fn op<T, S: Set<T>>(this: S, other: &S) -> S {
        this.intersection(other)
    }
}

#[allow(unused_variables)]
pub fn analyze<D, M, A, G, S, St, Idx, NodeIdx>(g: G, initial: St, dir: D, mode: M) -> A
where
    A: Analysis<State = St>,
    G: Graph<NodeIdx>,
    S: Set<Idx>,
    D: Direction,
    M: Mode,
    NodeIdx: Copy,
    Idx: Eq + Hash + Copy,
    St: self::State<Idx = Idx, NodeIdx = NodeIdx, Set = S>,
{
    analyze_kfn(g, initial, dir, mode, |a, k| a.difference(k))
}

#[allow(unused_variables)]
pub fn analyze_kfn<D, M, A, G, F, S, St, Idx, NodeIdx>(g: G, initial: St, dir: D, mode: M, kfn: F) -> A
where
    A: Analysis<State = St>,
    G: Graph<NodeIdx>,
    S: Set<Idx>,
    D: Direction,
    M: Mode,
    F: Fn(&S, &S) -> S,
    NodeIdx: Copy,
    Idx: Eq + Hash + Copy,
    St: self::State<Idx = Idx, NodeIdx = NodeIdx, Set = S>,
{
    let mut state = initial;
    let mut work_list = D::start(&g);

    while let Some(n) = work_list.pop_front() {
        let a = {
            let mut a = S::empty();
            for i in D::inputs(&g, n) {
                a = M::op(a, D::get_mut_b(&mut state, i));
            }
            a
        };

        let b = kfn(&a, state.kill(n)).union(state.gen(n));
        D::assign_a(&mut state, n, a);
        let state_b = D::get_mut_b(&mut state, n);

        if *state_b != b {
            *state_b = b;
            work_list.extend(D::affected(&g, n));
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
