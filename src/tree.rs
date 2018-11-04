use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub type NodeId = usize;

#[derive(Serialize, Deserialize)]
pub struct Tree<Node> {
    nodes: HashMap<NodeId, Node>,
    children: HashMap<NodeId, Vec<NodeId>>,
    parents: HashMap<NodeId, NodeId>,
    root_id: NodeId,
    next_id: NodeId
}

impl<Node> Tree<Node> {
    pub fn new() -> Self {
        Tree {
            nodes: HashMap::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
            root_id: 0,
            next_id: 1
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn all_ids(&self) -> impl Iterator<Item=NodeId> + '_ {
        self.nodes.keys().map(|id| *id)
    }

    pub fn all_nodes(&self) -> impl Iterator<Item=&Node> {
        self.nodes.values()
    }

    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    pub fn root(&self) -> Option<&Node> {
        self.nodes.get(&self.root_id)
    }

    pub fn root_mut(&mut self) -> Option<&mut Node> {
        self.nodes.get_mut(&self.root_id)
    }

    pub fn set_root(&mut self, node: Option<Node>) {
        if let Some(node) = node {
            self.nodes.insert(self.root_id, node);
        } else {
            self.nodes.remove(&self.root_id);
        }
    }

    pub fn child_ids(&self, id: NodeId) -> impl Iterator<Item=NodeId> + '_ {
        self.children.get(&id)
            .map(|vec| vec.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|id| *id)
    }

    pub fn children(&self, id: NodeId) -> impl Iterator<Item=&Node> {
        self.child_ids(id).map(move |id| self.node(id).unwrap())
    }

    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.parents.get(&id).cloned()
    }

    pub fn parent(&self, id: NodeId) -> Option<&Node> {
        self.parents.get(&id).cloned().and_then(move |id| self.node(id))
    }

    pub fn parent_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.parents.get(&id).cloned().and_then(move |id| self.node_mut(id))
    }

    pub fn insert_node(&mut self, parent_id: NodeId, node: Node) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        self.children.entry(parent_id).or_insert_with(Vec::new).push(id);
        self.parents.insert(id, parent_id);
        id
    }
}

impl<Node> Index<NodeId> for Tree<Node> {
    type Output = Node;

    fn index(&self, id: NodeId) -> &Node {
        self.node(id).unwrap()
    }
}

impl<Node> IndexMut<NodeId> for Tree<Node> {
    fn index_mut(&mut self, id: NodeId) -> &mut Node {
        self.node_mut(id).unwrap()
    }
}
