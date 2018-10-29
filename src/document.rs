use section::Section;
use petgraph::EdgeDirection;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use std::collections::HashMap;
use std::fmt;
use node::Node;
use headline::Headline;
use itertools::Itertools;
use parser::{Parser, Error};
use std::path::{Path, PathBuf};
use std::fs::File;

pub type NodeId = usize;

pub struct Document {
    pub path: Option<PathBuf>,
    pub title: String,
    pub section: Section,
    pub properties: HashMap<String, String>,
    graph: StableGraph<Node, ()>,
    sequential_ids: Vec<NodeId>,
}

impl Document {
    pub fn new(path: Option<PathBuf>) -> Self {
        Document {
            path,
            title: String::new(),
            section: Section::new(),
            properties: HashMap::new(),
            graph: StableGraph::new(),
            sequential_ids: vec![],
        }
    }

    pub fn open_file(path: &Path) -> Result<Self, Error> {
        let file = File::open(path)
            .map_err(Error::IoError)?;
        Document::parse(Some(path.into()), Parser::from_file(file))
    }

    pub fn from_string(source: &str) -> Result<Self, Error> {
        Document::parse(None, Parser::from_string(source))
    }

    fn parse(path: Option<PathBuf>, mut parser: Parser) -> Result<Self, Error> {
        use planning::Planning;
        use element::Element;
        use headline::Headline;
        use drawer::Drawer;

        let todo_keywords = vec!["TODO".to_string(), "DONE".to_string()];

        let mut document = Document::new(path);
        let mut current_id = document.root_id();

        while let Some(line) = parser.next()? {
            if let Some(headline) = Headline::parse(&line, &todo_keywords) {
                current_id = Some(document.add_new_node(current_id, headline));
            } else if let Some(drawer) = Drawer::parse(&line, &mut parser)? {
                if let Some(properties) = drawer.as_properties() {
                    if let Some(current_id) = current_id {
                        // TODO: Must property drawers come immediately after the headline
                        // and/or planning info?
                        document.node_mut(current_id).unwrap().properties = properties;
                    } else {
                        // TODO: Are property drawers valid outside of a headlien?
                        document.properties.extend(properties);
                    }
                } else {
                    document.section_mut(current_id).unwrap().add_drawer(drawer);
                }
            } else if let Some(planning) = Planning::parse(&line)? {
                if let Some(current_id) = current_id {
                    document.node_mut(current_id).unwrap().set_planning(planning, line);
                } else {
                    println!("WARNING: planning info found above first headline");
                    document.section_mut(current_id).unwrap().add_line(line);
                }
            } else if let Some(element) = Element::parse_greater(&line, &mut parser)? {
                document.section_mut(current_id).unwrap().elements.push(element);
            } else {
                document.section_mut(current_id).unwrap().add_line(line);
            }
        }

        Ok(document)
    }

    pub fn root_id(&self) -> Option<NodeId> {
        None
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.graph.node_weight(NodeIndex::new(id))
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.graph.node_weight_mut(NodeIndex::new(id))
    }

    pub fn all_ids(&self) -> Vec<NodeId> {
        self.graph.node_indices().map(|index| index.index()).collect()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item=&Node> {
        self.all_ids().into_iter().map(move |id| self.node(id).unwrap())
    }

    pub fn child_ids(&self) -> Vec<NodeId> {
        self.children_of(self.root_id())
    }

    pub fn children(&self) -> impl Iterator<Item=&Node> {
        self.children_of(self.root_id()).into_iter().map(move |id| self.node(id).unwrap())
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

        if parent_id.is_some() && self.node(parent_id.unwrap()).is_none() {
            println!("WARNING: Node not found: {}", parent_id.unwrap());
            return None
        }

        while parent_id.is_some() && self.node(parent_id.unwrap()).unwrap().indent >= indent {
            parent_id = self.parent_of(parent_id.unwrap());
        }

        if let Some(parent_id) = parent_id {
            let expected_indent = self.node(parent_id).unwrap().indent + 1;
            if indent > expected_indent {
                println!("WARNING: Indent is too deep: {} > {}", indent, expected_indent);
            }
        } else if indent > 1 {
            println!("WARNING: Indent is too deep: {} > 0", indent);
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

    pub fn section(&self, id: Option<NodeId>) -> Option<&Section> {
        if let Some(id) = id {
            self.node(id).map(|node| &node.section)
        } else {
            Some(&self.section)
        }
    }

    pub fn section_mut(&mut self, id: Option<NodeId>) -> Option<&mut Section> {
        if let Some(id) = id {
            self.node_mut(id).map(|node| &mut node.section)
        } else {
            Some(&mut self.section)
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.all_nodes().join("\n"))
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Document")
            .field("title", &self.title)
            .finish()
    }
}
