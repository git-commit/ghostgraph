# ghostgraph

> A rust graph implementation using Ghost Cells to enable internal mutability

## Description of our graph data structure's interface

### Node Structure

- incoming edges as vectors via GhostCell<T> immutable reference
- outgoing edges as vectors via GhostCell<T> immutable reference
