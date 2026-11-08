// Same algorithm and api as petgraph's simple_fast (Cooper, Harvey, Kennedy), but
// everything is indexed by NodeIndex instead of going through std's SipHash maps,
// which made dominators one of the hotter things on profiles. The dominator tree is
// unique, so results are identical.

use petgraph::{
    stable_graph::NodeIndex,
    visit::{DfsPostOrder, IntoNeighbors, NodeIndexable, Visitable, Walker},
};

const UNDEFINED: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct Dominators {
    root: NodeIndex,
    // immediate dominator by node index, NodeIndex::end() for unreachable nodes
    idom: Vec<NodeIndex>,
}

impl Dominators {
    pub fn root(&self) -> NodeIndex {
        self.root
    }

    fn idom_of(&self, node: NodeIndex) -> Option<NodeIndex> {
        self.idom
            .get(node.index())
            .copied()
            .filter(|&d| d != NodeIndex::end())
    }

    pub fn immediate_dominator(&self, node: NodeIndex) -> Option<NodeIndex> {
        if node == self.root {
            None
        } else {
            self.idom_of(node)
        }
    }

    pub fn strict_dominators(&self, node: NodeIndex) -> Option<DominatorsIter<'_>> {
        self.idom_of(node).map(|_| DominatorsIter {
            dominators: self,
            node: self.immediate_dominator(node),
        })
    }

    pub fn dominators(&self, node: NodeIndex) -> Option<DominatorsIter<'_>> {
        self.idom_of(node).map(|_| DominatorsIter {
            dominators: self,
            node: Some(node),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DominatorsIter<'a> {
    dominators: &'a Dominators,
    node: Option<NodeIndex>,
}

impl Iterator for DominatorsIter<'_> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        let next = self.node.take();
        if let Some(next) = next {
            self.node = self.dominators.immediate_dominator(next);
        }
        next
    }
}

pub fn simple_fast<G>(graph: G, root: NodeIndex) -> Dominators
where
    G: IntoNeighbors<NodeId = NodeIndex> + Visitable + NodeIndexable,
{
    let post_order = DfsPostOrder::new(graph, root)
        .iter(graph)
        .collect::<Vec<_>>();
    let length = post_order.len();
    debug_assert_eq!(post_order.last(), Some(&root));

    let mut post_order_idx = vec![UNDEFINED; graph.node_bound()];
    for (idx, node) in post_order.iter().enumerate() {
        post_order_idx[node.index()] = idx;
    }
    let mut predecessors = vec![Vec::new(); length];
    for (idx, &node) in post_order.iter().enumerate() {
        for successor in graph.neighbors(node) {
            predecessors[post_order_idx[successor.index()]].push(idx);
        }
    }

    let mut dominators = vec![UNDEFINED; length];
    dominators[length - 1] = length - 1;
    let mut changed = true;
    while changed {
        changed = false;
        for idx in (0..length - 1).rev() {
            let mut processed = predecessors[idx]
                .iter()
                .copied()
                .filter(|&p| dominators[p] != UNDEFINED);
            let first = processed
                .next()
                .expect("the root dominates itself, so some predecessor is already processed");
            let new_idom = processed.fold(first, |a, b| intersect(&dominators, a, b));
            if new_idom != dominators[idx] {
                dominators[idx] = new_idom;
                changed = true;
            }
        }
    }

    let mut idom = vec![NodeIndex::end(); graph.node_bound()];
    for (idx, node) in post_order.iter().enumerate() {
        idom[node.index()] = post_order[dominators[idx]];
    }
    Dominators { root, idom }
}

fn intersect(dominators: &[usize], mut finger1: usize, mut finger2: usize) -> usize {
    while finger1 != finger2 {
        if finger1 < finger2 {
            finger1 = dominators[finger1];
        } else {
            finger2 = dominators[finger2];
        }
    }
    finger1
}
