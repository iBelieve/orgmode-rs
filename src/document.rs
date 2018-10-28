use section::Section;
use chrono::prelude::*;
use petgraph::EdgeDirection;
use petgraph::stable_graph::{NodeIndex, NodeIndices, StableGraph};
use std::collections::HashMap;
use std::fmt;
use node::Node;
use headline::Headline;

pub type NodeId = usize;

pub struct Document {
    pub title: String,
    pub section: Section,
    pub properties: HashMap<String, String>,
    graph: StableGraph<Node, ()>,
    sequential_ids: Vec<NodeId>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            title: String::new(),
            section: Section::new(),
            properties: HashMap::new(),
            graph: StableGraph::new(),
            sequential_ids: vec![],
        }
    }

    pub fn root_id(&self) -> Option<NodeId> {
        None
    }

    pub fn node(&self, id: NodeId) -> &Node {
        self.graph.node_weight(NodeIndex::new(id)).unwrap()
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.graph.node_weight_mut(NodeIndex::new(id)).unwrap()
    }

    pub fn all_ids(&self) -> Vec<NodeId> {
        self.graph.node_indices().map(|index| index.index()).collect()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item=&Node> {
        self.all_ids().into_iter().map(move |id| self.node(id))
    }

    pub fn child_ids(&self) -> Vec<NodeId> {
        self.children_of(self.root_id())
    }

    pub fn children(&self) -> impl Iterator<Item=&Node> {
        self.children_of(self.root_id()).into_iter().map(move |id| self.node(id))
    }

    pub fn children_of(&self, id: Option<NodeId>) -> Vec<NodeId> {
        let mut children: Vec<NodeId> = if let Some(id) = id {
            self.graph
                .neighbors_directed(NodeIndex::new(id), EdgeDirection::Outgoing)
                .map(|index| index.index())
                .collect()
        } else {
            self.graph
                .externals(EdgeDirection::Incoming)
                .map(|index| index.index())
                .collect()
        };
        children.sort_by_key(|node_id| self.index_of(*node_id));
        children
    }

    fn index_of(&self, id: NodeId) -> usize {
        if self.sequential_ids.last() == Some(&id) {
            self.sequential_ids.len() - 1
        } else {
            self.sequential_ids.iter()
                .position(|an_id| *an_id == id)
                .expect("Unable to find ID in sequence")
        }
    }

    pub fn id_after(&self, id: NodeId) -> Option<NodeId> {
        self.sequential_ids.get(self.index_of(id) + 1).cloned()
    }

    pub fn id_before(&self, id: NodeId) -> Option<NodeId> {
        self.sequential_ids.get(self.index_of(id) - 1).cloned()
    }

    pub fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        let mut parents = self.graph.neighbors_directed(NodeIndex::new(id), EdgeDirection::Incoming);

        if let Some(parent) = parents.next() {
            assert_eq!(parents.next(), None);
            Some(parent.index())
        } else {
            None
        }
    }

    pub fn find_parent(&self, current_id: Option<NodeId>, indent: u16) -> Option<NodeId> {
        let mut parent_id = current_id;

        while parent_id.is_some() && self.node(parent_id.unwrap()).indent >= indent {
            parent_id = self.parent_of(parent_id.unwrap());
        }

        if let Some(parent_id) = parent_id {
            if indent > self.node(parent_id).indent + 1 {
                println!("WARNING: Indent is too deep")
            }
        } else if indent > 0 {
            println!("WARNING: Indent is too deep")
        }

        parent_id
    }

    pub fn add_new_node(&mut self, current_id: Option<NodeId>, headline: Headline) -> NodeId {
        let indent = headline.indent;
        let parent_id = self.find_parent(current_id, indent);
        let new_id = self.graph.add_node(Node::from_headline(headline)).index();
        let new_index = self.next_index_after(current_id);
        if let Some(parent_id) = parent_id {
            self.graph.add_edge(NodeIndex::new(parent_id), NodeIndex::new(new_id), ());
        }
        self.sequential_ids.insert(new_index, new_id);
        new_id
    }

    pub fn next_index_after(&self, id: Option<NodeId>) -> usize {
        if self.sequential_ids.last() == id.as_ref() || id == None {
            self.sequential_ids.len()
        } else {
            let id = id.unwrap();
            let last_id = self.children_of(Some(id)).last().cloned().unwrap_or(id);

            self.index_of(last_id) + 1
        }
    }

    pub fn section(&self, id: Option<NodeId>) -> &Section {
        if let Some(id) = id {
            &self.node(id).section
        } else {
            &self.section
        }
    }

    pub fn section_mut(&mut self, id: Option<NodeId>) -> &mut Section {
        if let Some(id) = id {
            &mut self.node_mut(id).section
        } else {
            &mut self.section
        }
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Document")
            .field("title", &self.title)
            .finish()
    }
}
