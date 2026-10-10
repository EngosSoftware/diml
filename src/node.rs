//! # Tree node implementation

use std::fmt::Write;

/// Tree node.
#[derive(Debug, Clone)]
pub struct Node {
  /// Indentation level of the node.
  /// Root (virtual) node has level `0`.
  /// Top level nodes in parsed document have level `1`.
  level: usize,
  /// Name delimiter.
  /// Delimiter character as defined in parsed document.
  /// Root (virtual) node hss no delimiter.
  delimiter: Option<char>,
  /// The name of the node.
  /// Node name as defined in parsed document without delimiter.
  /// Root (virtual) node has an empty name.
  name: String,
  /// The content of the node.
  /// Node content as defined in parsed document.
  /// Root (virtual) node has an empty content.
  content: String,
  /// Child nodes.
  /// A list of all child nodes of this node.
  children: Vec<Node>,
}

impl Node {
  /// Creates a root node.
  pub(crate) fn root() -> Self {
    Self {
      level: 0,
      delimiter: None,
      name: "".to_string(),
      content: "".to_string(),
      children: vec![],
    }
  }

  /// Creates a new node.
  pub(crate) fn new(level: usize, delimiter: Option<char>, name: String, content: String) -> Self {
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
  pub fn delimiter(&self) -> Option<char> {
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
  pub fn document(&self, indentation: usize, ch: char) -> String {
    let mut buffer = String::new();
    if self.level > 0 {
      let indentation = if self.level > 1 {
        ch.to_string().repeat((self.level - 1) * indentation)
      } else {
        "".to_string()
      };
      let _ = write!(&mut buffer, "{}{}{}{}", indentation, self.delimiter.unwrap_or_default(), self.name, self.content);
    }
    for child in &self.children {
      let _ = write!(&mut buffer, "{}", child.document(indentation, ch));
    }
    buffer
  }
}
