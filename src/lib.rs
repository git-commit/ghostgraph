use std::{collections::HashSet, ptr::NonNull};

use ghost_cell::{GhostCell, GhostToken};

pub struct Node<'brand, 'graph> {
    incoming: Vec<&'graph GhostCell<'brand, Node<'brand, 'graph>>>,
    outgoing: Vec<&'graph GhostCell<'brand, Node<'brand, 'graph>>>,
}

impl<'brand, 'graph> Node<'brand, 'graph> {
    pub fn new() -> Self {
        Self {
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

pub fn add_edge<'brand, 'graph>(
    node1: &'graph GhostCell<'brand, Node<'brand, 'graph>>,
    node2: &'graph GhostCell<'brand, Node<'brand, 'graph>>,
    token: &mut GhostToken<'brand>,
) {
    node1.borrow_mut(token).outgoing.push(node2);
    node2.borrow_mut(token).incoming.push(node1);
}

pub fn bfs<'brand, 'graph>(
    node1: &'graph GhostCell<'brand, Node<'brand, 'graph>>,
    token: &GhostToken<'brand>,
    mut closure: impl FnMut(&'graph GhostCell<Node<'brand, 'graph>>, &GhostToken<'brand>),
) {
    let mut visited: HashSet<NonNull<GhostCell<'brand, Node<'brand, 'graph>>>> = HashSet::new();

    let mut node_stack = vec![node1];
    visited.insert(NonNull::from(node1));

    loop {
        let Some(current) = node_stack.pop() else {
            break;
        };

        closure(current, &token);
        let node = current.borrow(&token);
        for pardner in node.outgoing.iter() {
            if visited.insert(NonNull::from(*pardner)) {
                node_stack.push(pardner);
            }
        }
    }
}

pub fn count<'brand, 'graph>(
    start: &'graph GhostCell<'brand, Node<'brand, 'graph>>,
    token: &GhostToken<'brand>,
) -> usize {
    let mut count = 0;
    bfs(&start, &token, |_, _| count += 1);
    count
}

pub fn iter_edges<'brand, 'graph, 'token>(
    start: &'graph GhostCell<'brand, Node<'brand, 'graph>>,
    token: &'token GhostToken<'brand>,
) -> impl Iterator<
    Item = (
        &'graph GhostCell<'brand, Node<'brand, 'graph>>,
        &'graph GhostCell<'brand, Node<'brand, 'graph>>,
        &'token GhostToken<'brand>,
    ),
> {
    let mut node_stack = vec![start];
    let mut visited = HashSet::new();
    let mut i = 0;

    visited.insert(NonNull::from(start));
    core::iter::from_fn(move || {
        loop {
            let &current = node_stack.last()?;
            let node = current.borrow(token);
            if let Some(&pardner) = node.outgoing.get(i) {
                i += 1;
                return Some((current, pardner, token));
            } else {
                i = 0;
                // Matches last before.
                let _ = node_stack.pop()?;
                for pardner in node.outgoing.iter() {
                    if visited.insert(NonNull::from(*pardner)) {
                        node_stack.push(pardner);
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest_derive::Arbitrary;
    use proptest::prelude::*;

    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Hash)]
    enum Operation {
        AddEdge(usize, usize),
    }

    proptest!{
        #[test]
        fn test_arbitrary_graph(num_nodes in 1..42usize, ops in prop::collection::vec(any::<Operation>(), 0..23)) {
            GhostToken::new(|mut token| {
                let nodes: Vec<_> = (0..num_nodes)
                    .map(|_| GhostCell::new(Node::new()))
                    .collect();

                for op in &ops {
                    match op {
                        Operation::AddEdge(lhs, rhs) => {
                            add_edge(
                                &nodes[lhs % num_nodes],
                                &nodes[rhs % num_nodes],
                                &mut token,
                            )
                        }
                    }
                }

                // Reachable graph at most all nodes.
                assert!(count(&nodes[0], &token) <= num_nodes);
                // Does not panic, and no more edges than we had.
                assert!(iter_edges(&nodes[0], &token).count() <= ops.len());
            })
        }
    }

    #[test]
    fn test_add_edge() {
        GhostToken::new(|mut token| {
            let node1 = GhostCell::new(Node::new());
            let node2 = GhostCell::new(Node::new());
            let node3 = GhostCell::new(Node::new());
            let node4 = GhostCell::new(Node::new());

            add_edge(&node1, &node2, &mut token);
            add_edge(&node2, &node3, &mut token);
            add_edge(&node3, &node4, &mut token);

            let count = count(&node1, &token);
            assert_eq!(count, 4);
        })
    }

    #[test]
    fn iterate_recursive() {
        GhostToken::new(|mut token| {
            let node1 = GhostCell::new(Node::new());
            let node2 = GhostCell::new(Node::new());

            add_edge(&node1, &node2, &mut token);
            add_edge(&node2, &node1, &mut token);

            let count = count(&node1, &token);

            assert_eq!(count, 2);
        });
    }

    #[test]
    fn test_count() {
        GhostToken::new(|mut token| {
            let node1 = GhostCell::new(Node::new());
            let node2 = GhostCell::new(Node::new());
            let node3 = GhostCell::new(Node::new());

            add_edge(&node1, &node2, &mut token);
            add_edge(&node2, &node1, &mut token);
            add_edge(&node1, &node3, &mut token);

            assert_eq!(count(&node1, &token), 3);
        });
    }

    #[test]
    fn iter_edged_by_counting_because_rahix() {
        GhostToken::new(|mut token| {
            let node1 = GhostCell::new(Node::new());
            let node2 = GhostCell::new(Node::new());
            let node3 = GhostCell::new(Node::new());

            add_edge(&node1, &node2, &mut token);
            add_edge(&node2, &node1, &mut token);
            add_edge(&node2, &node1, &mut token);
            add_edge(&node2, &node1, &mut token);
            add_edge(&node1, &node3, &mut token);
            add_edge(&node3, &node3, &mut token);
            add_edge(&node3, &node1, &mut token);

            for (lhs, rhs, _) in iter_edges(&node1, &token) {
                eprintln!("{:p} -> {:p}", lhs, rhs);
            }

            let count = iter_edges(&node1, &token).count();
            assert_eq!(count, 7);
        })
    }

    #[test]
    fn iter_edges_zero_edges() {
        GhostToken::new(|token| {
            let start = GhostCell::new(Node::new());
            let count = iter_edges(&start, &token).count();
            assert_eq!(count, 0);
        })
    }
}
