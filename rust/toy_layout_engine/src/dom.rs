use std::collections::{HashMap, HashSet};

/// The DOM is a tree of nodes.
#[derive(Debug)]
pub struct Node {
    /// data common to all nodes:
    pub children: Vec<Node>,

    /// data specific to each node type:
    pub node_type: NodeType,
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

/// An element includes a tag name and any number of attributes, which can be stored as a map from names to values.
#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    /// The class attribute can contain multiple class names separated by spaces, which we return in a hash table.
    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}

pub type AttrMap = HashMap<String, String>;

pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children: children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}
