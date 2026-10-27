//! # Tree node implementation

/// Tree node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Indentation level of the node.
    /// Root (virtual) node has level `0`.
    /// Top level nodes in parsed document have level `1`.
    level: usize,
    /// Name delimiter.
    /// Delimiter character as defined in parsed document.
    /// Root (virtual) node has no delimiter.
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
    children: Vec<Self>,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or(2);
        if let Some(delimiter) = self.delimiter {
            let indent = self.level.saturating_sub(1) * width;
            write!(f, "{:indent$}{delimiter}{}{}", "", self.name, self.content)?;
        }
        for child in &self.children {
            write!(f, "{child:width$}")?;
        }
        Ok(())
    }
}

impl Node {
    /// Creates a root node.
    pub(crate) const fn root() -> Self {
        Self {
            level: 0,
            delimiter: None,
            name: String::new(),
            content: String::new(),
            children: vec![],
        }
    }

    /// Creates a new node.
    pub(crate) const fn new(
        level: usize,
        delimiter: Option<char>,
        name: String,
        content: String,
    ) -> Self {
        Self {
            level,
            delimiter,
            name,
            content,
            children: vec![],
        }
    }

    /// Adds a child node at the end of the children list.
    pub(crate) fn add_child(&mut self, node: Self) {
        self.children.push(node);
    }

    /// Returns the indentation level of the node.
    #[must_use]
    pub const fn level(&self) -> usize {
        self.level
    }

    /// Returns the delimiter of the node.
    #[must_use]
    pub const fn delimiter(&self) -> Option<char> {
        self.delimiter
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

    /// Returns the node text (trimmed content).
    #[must_use]
    pub fn text(&self) -> &str {
        self.content.trim()
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

    /// Returns an iterator over all child nodes.
    pub fn children(&self) -> impl Iterator<Item = &Self> {
        self.children.iter()
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

    /// Returns the number of child nodes.
    #[must_use]
    pub const fn child_count(&self) -> usize {
        self.children.len()
    }
}
