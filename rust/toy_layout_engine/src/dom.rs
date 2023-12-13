use std::collections::HashMap;

/// The DOM is a tree of nodes.
pub struct Node {
    /// data common to all nodes:
    pub children: Vec<Node>,

    /// data specific to each node type:
    pub node_type: NodeType,
}

pub enum NodeType {
    Text(String),
    Element(ElementData),
}

/// An element includes a tag name and any number of attributes, which can be stored as a map from names to values.
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
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

