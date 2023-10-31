
pub struct Node<'a> {
    incoming: Vec<&'a ghost_cell::GhostCell<'a, Node<'a>>>,
    outgoing: Vec<&'a ghost_cell::GhostCell<'a, Node<'a>>>,
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Self {
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

pub fn add_edge<'a>(
    node1: &'a ghost_cell::GhostCell<'a, Node<'a>>, 
    node2: &'a ghost_cell::GhostCell<'a, Node<'a>>, 
    token: &mut ghost_cell::GhostToken<'a>) 
{
    node1.borrow_mut(token).outgoing.push(node2);
    node2.borrow_mut(token).incoming.push(node1);
}