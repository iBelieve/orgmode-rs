use headline::Headline;
use itertools::Itertools;
use node::{Node, NodeId};
use parser::Parser;
use section::Section;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Error as IoError;
use std::iter::repeat;
use std::path::{Path, PathBuf};
use timestamp::{Date, Duration, Timestamp};
use tree::Tree;

pub type DocumentId = usize;

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub path: Option<PathBuf>,
    pub title: String,
    pub section: Section,
    pub properties: HashMap<String, String>,
    #[serde(flatten)]
    tree: Tree<Node>,
}

impl Document {
    pub fn new(path: Option<PathBuf>) -> Self {
        Document {
            id: 0,
            path,
            title: String::new(),
            section: Section::new(),
            properties: HashMap::new(),
            tree: Tree::new(),
        }
    }

    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, IoError> {
        let path = path.as_ref();
        let file = File::open(path)?;
        Document::parse(Some(path.into()), Parser::from_file(file))
    }

    pub fn from_string(source: &str) -> Self {
        Document::parse(None, Parser::from_string(source)).unwrap()
    }

    fn parse(path: Option<PathBuf>, mut parser: Parser) -> Result<Self, IoError> {
        use drawer::Drawer;
        use element::Element;
        use headline::Headline;
        use planning::Planning;

        let todo_keywords = vec![
            "TODO".to_string(),
            "IN-PROGRESS".to_string(),
            "DONE".to_string(),
        ];

        let mut document = Document::new(path);
        let mut current_id = None;

        while let Some(line) = parser.next() {
            if let Some(headline) = Headline::parse(&line, &todo_keywords) {
                current_id = Some(document.add_new_node(current_id, headline));
            } else if let Some(drawer) = Drawer::parse(&line, &mut parser) {
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
            } else if let Some(planning) = Planning::parse(&line) {
                if let Some(current_id) = current_id {
                    document
                        .node_mut(current_id)
                        .unwrap()
                        .set_planning(planning, &line);
                } else {
                    org_warning!("planning info found above first headline");
                    document.section_mut(current_id).unwrap().add_line(&line);
                }
            } else if let Some(element) = Element::parse(&line, &mut parser) {
                document
                    .section_mut(current_id)
                    .unwrap()
                    .elements
                    .push(element);
            } else {
                document.section_mut(current_id).unwrap().add_line(&line);
            }
        }

        if let Some(error) = parser.io_error {
            Err(error)
        } else {
            Ok(document)
        }
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.tree.node(id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.tree.node_mut(id)
    }

    pub fn all_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.tree.all_ids()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &Node> {
        self.tree.all_nodes()
    }

    pub fn root_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.tree.child_ids(self.tree.root_id())
    }

    pub fn roots(&self) -> impl Iterator<Item = &Node> {
        self.tree.children(self.tree.root_id())
    }

    pub fn child_ids(&self, id: Option<NodeId>) -> impl Iterator<Item = NodeId> + '_ {
        self.tree.child_ids(id.unwrap_or(self.tree.root_id()))
    }

    pub fn children(&self, id: Option<NodeId>) -> impl Iterator<Item = &Node> {
        self.tree.children(id.unwrap_or(self.tree.root_id()))
    }

    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.tree
            .parent_id(id)
            .filter(|id| id != &self.tree.root_id())
    }

    pub fn parent(&self, id: NodeId) -> Option<&Node> {
        self.tree.parent(id)
    }

    pub fn parent_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.tree.parent_mut(id)
    }

    pub fn find_parent(&self, current_id: Option<NodeId>, indent: u16) -> Option<NodeId> {
        let mut parent_id = current_id;

        if parent_id.is_some() && self.node(parent_id.unwrap()).is_none() {
            org_warning!("Node not found: {}", parent_id.unwrap());
            return None;
        }

        while parent_id.is_some() && self.node(parent_id.unwrap()).unwrap().indent >= indent {
            parent_id = self.parent_id(parent_id.unwrap());
        }

        if let Some(parent_id) = parent_id {
            let expected_indent = self.node(parent_id).unwrap().indent + 1;
            if indent > expected_indent {
                org_warning!("Indent is too deep: {} > {}", indent, expected_indent);
            }
        } else if indent > 1 {
            org_warning!("Indent is too deep: {} > 0", indent);
        }

        parent_id
    }

    pub fn add_new_node(&mut self, current_id: Option<NodeId>, headline: Headline) -> NodeId {
        let indent = headline.indent;
        let parent_id = self
            .find_parent(current_id, indent)
            .unwrap_or(self.tree.root_id());
        let new_id = self
            .tree
            .insert_node(parent_id, Node::from_headline(headline));
        self.tree[new_id].id = new_id;
        new_id
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

    pub fn nodes_for_date<'a>(
        &'a self,
        date: &'a Date,
    ) -> impl Iterator<Item = (Timestamp, &'a Node)> {
        self.all_nodes()
            .flat_map(move |node| node.timestamps_for_date(date).zip(repeat(node)))
    }

    pub fn nodes_past_scheduled(&self) -> impl Iterator<Item = &Node> {
        self.all_nodes()
            .filter(move |node| node.is_past_scheduled())
    }

    pub fn nodes_past_deadline(&self) -> impl Iterator<Item = &Node> {
        self.all_nodes().filter(move |node| node.is_past_deadline())
    }

    // TODO: Support extending parent properties via NAME+
    pub fn node_property(&self, node_id: NodeId, name: &str) -> Option<&str> {
        let mut node_id = Some(node_id);

        while let Some(id) = node_id {
            if let Some(node) = self.node(id) {
                if let Some(property) = node.properties.get(name) {
                    return Some(property);
                }
            }
            node_id = self.parent_id(id);
        }

        None
    }

    pub fn node_category(&self, node_id: NodeId) -> Option<&str> {
        if let Some(category) = self.node_property(node_id, "CATEGORY") {
            Some(category)
        } else if let Some(ref path) = self.path {
            path.file_stem().and_then(|stem| stem.to_str())
        } else {
            None
        }
    }

    pub fn node_time_spent(&self, node_id: NodeId) -> Duration {
        let node_time = self
            .node(node_id)
            .map(|node| node.logbook().time_spent())
            .unwrap_or_else(Duration::zero);
        let children_time = self
            .child_ids(Some(node_id))
            .map(|child_id| self.node_time_spent(child_id))
            .fold(Duration::zero(), |total, duration| total + duration);

        node_time + children_time
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.section)?;
        if !self.section.is_empty() && !self.tree.is_empty() {
            write!(f, "\n")?;
        }
        write!(f, "{}", self.all_nodes().join("\n"))
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Document")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("title", &self.title)
            .finish()
    }
}
