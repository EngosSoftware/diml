//! # Tree node implementation

use crate::defs::*;
use std::fmt::Write;

const ROOT_LEVEL: usize = 0;
const ROOT_DELIMITER: char = NULL;
const ROOT_NAME: &str = "root";
const ROOT_CONTENT: &str = "";

/// Tree node.
#[derive(Debug, Clone)]
pub struct Node {
  /// Indentation level of the node.
  /// Root note has level `0`. Top level nodes have value `1`.
  level: usize,
  /// Name delimiter.
  /// Original name delimiter as defined in the parsed document.
  delimiter: char,
  /// The name of the node.
  /// Original name as defined in the parsed document but without delimiter.
  name: String,
  /// The content of the node.
  /// Original content as defined in the parsed document.
  content: String,
  /// Child nodes.
  /// A list of all child nodes in the document tree.
  children: Vec<Node>,
}

impl Node {
  /// Creates a root node.
  pub(crate) fn root() -> Self {
    Self {
      level: ROOT_LEVEL,
      delimiter: ROOT_DELIMITER,
      name: ROOT_NAME.to_string(),
      content: ROOT_CONTENT.to_string(),
      children: vec![],
    }
  }

  /// Returns `true` when node is a root.
  pub(crate) fn is_root(&self) -> bool {
    self.level == ROOT_LEVEL && self.delimiter == ROOT_DELIMITER && self.name == ROOT_NAME && self.content == ROOT_CONTENT
  }

  /// Creates a new node.
  pub(crate) fn new(level: usize, delimiter: char, name: String, content: String) -> Self {
    Self {
      level,
      delimiter,
      name,
      content,
      children: vec![],
    }
  }

  /// Adds a child node at the end of the children list.
  pub(crate) fn add_child(&mut self, node: Node) {
    self.children.push(node);
  }

  /// Returns the indentation level of the node.
  pub fn level(&self) -> usize {
    self.level
  }

  /// Returns the delimiter of the node.
  pub fn delimiter(&self) -> char {
    self.delimiter
  }

  /// Returns the node name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the node content.
  pub fn content(&self) -> &str {
    &self.content
  }

  /// Returns the node text.
  /// Node text is a trimmed node content.
  pub fn text(&self) -> &str {
    self.content.trim()
  }

  /// Returns the first child node having the specified name.
  pub fn first_with_name(&self, name: impl AsRef<str>) -> Option<&Node> {
    self.children.iter().find(|node| node.name == name.as_ref())
  }

  /// Returns the last child node having the specified name.
  pub fn last_with_name(&self, name: impl AsRef<str>) -> Option<&Node> {
    self.children.iter().rev().find(|node| node.name == name.as_ref())
  }

  /// Returns an iterator over all child nodes.
  pub fn children(&self) -> impl Iterator<Item = &Node> {
    self.children.iter()
  }

  /// Returns an iterator over child nodes that have the specified name.
  pub fn with_name(&self, name: impl AsRef<str>) -> impl Iterator<Item = &Node> {
    self.children.iter().filter(move |node| node.name == name.as_ref())
  }

  /// Returns an iterator over child nodes that have any of the specified names.
  pub fn with_names<'a>(&'a self, names: &'a [impl AsRef<str>]) -> impl Iterator<Item = &'a Node> {
    let names = names.iter().map(|name| name.as_ref()).collect::<Vec<&str>>();
    self.children.iter().filter(move |node| names.contains(&node.name()))
  }

  /// Returns an iterator over child nodes, excluding those with the specified name.
  pub fn excluding_name(&self, name: impl AsRef<str>) -> impl Iterator<Item = &Node> {
    self.children.iter().filter(move |node| node.name != name.as_ref())
  }

  /// Returns an iterator over child nodes, excluding those with any of the specified names.
  pub fn excluding_names<'a>(&'a self, names: &'a [impl AsRef<str>]) -> impl Iterator<Item = &'a Node> {
    let names = names.iter().map(|name| name.as_ref()).collect::<Vec<&str>>();
    self.children.iter().filter(move |node| !names.contains(&node.name()))
  }

  /// Returns the number of child nodes.
  pub fn child_count(&self) -> usize {
    self.children.len()
  }

  /// Returns a document starting from this node.
  pub fn document(&self, indent: usize, ch: char) -> String {
    let mut buffer = String::new();
    if !self.is_root() {
      let indentation = if self.level > 1 {
        ch.to_string().repeat((self.level - 1) * indent)
      } else {
        "".to_string()
      };
      let _ = write!(&mut buffer, "{}{}{}{}", indentation, self.delimiter, self.name, self.content);
    }
    for child in &self.children {
      let _ = write!(&mut buffer, "{}", child.document(indent, ch));
    }
    buffer
  }
}
