//! # Node implementation

/// Tree node.
#[derive(Debug)]
pub struct Node {
    /// Line number where the node starts.
    line: usize,
    /// Node name.
    name: String,
    /// Node content.
    content: String,
    /// Child nodes.
    children: Vec<Self>,
}

impl Node {
    /// Creates a new node.
    pub(crate) const fn new(line: usize, name: String, content: String) -> Self {
        Self {
            line,
            name,
            content,
            children: vec![],
        }
    }

    /// Returns the line number where the node starts in input.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the node name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the node content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Adds a child node at the end of the children list.
    pub(crate) fn add_child(&mut self, node: Self) {
        self.children.push(node);
    }

    /// Returns an iterator over all child nodes.
    pub fn children(&self) -> std::slice::Iter<'_, Self> {
        self.children.iter()
    }

    /// Returns the first child node having the specified name.
    pub fn first_with_name(&self, name: impl AsRef<str>) -> Option<&Self> {
        self.children.iter().find(|node| node.name == name.as_ref())
    }

    /// Returns the last child node having the specified name.
    pub fn last_with_name(&self, name: impl AsRef<str>) -> Option<&Self> {
        self.children
            .iter()
            .rev()
            .find(|node| node.name == name.as_ref())
    }

    /// Returns an iterator over child nodes that have the specified name.
    pub fn with_name(&self, name: impl AsRef<str>) -> impl Iterator<Item = &Self> {
        self.children
            .iter()
            .filter(move |node| node.name == name.as_ref())
    }

    /// Returns an iterator over child nodes that have any of the specified names.
    pub fn with_names<'a>(
        &'a self,
        names: &'a [impl AsRef<str>],
    ) -> impl Iterator<Item = &'a Self> {
        self.children
            .iter()
            .filter(|node| names.iter().any(|name| name.as_ref() == node.name))
    }

    /// Returns an iterator over child nodes, excluding those with the specified name.
    pub fn excluding_name(&self, name: impl AsRef<str>) -> impl Iterator<Item = &Self> {
        self.children
            .iter()
            .filter(move |node| node.name != name.as_ref())
    }

    /// Returns an iterator over child nodes, excluding those with any of the specified names.
    pub fn excluding_names<'a>(
        &'a self,
        names: &'a [impl AsRef<str>],
    ) -> impl Iterator<Item = &'a Self> {
        self.children
            .iter()
            .filter(|node| !names.iter().any(|name| name.as_ref() == node.name))
    }
}
