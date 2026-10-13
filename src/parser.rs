//! # Parser implementation

use crate::Node;
use crate::errors::*;
use crate::recognizer::{Item, recognize};

/// Parses input text.
pub fn parse(input: &str) -> Result<Node> {
  let mut nodes = vec![];
  let mut stack = vec![];
  let mut multiplier = 0;
  for Item {
    indent_line: line,
    indent_width: indentation,
    delimiter,
    name,
    content,
  } in recognize(input)?
  {
    if multiplier == 0 && indentation > 0 {
      multiplier = indentation;
    }
    if multiplier > 0 && !indentation.is_multiple_of(multiplier) {
      return Err(err_invalid_indentation(indentation, multiplier, line, indentation));
    }
    let level = indentation.checked_div(multiplier).unwrap_or(0) + 1;
    let previous_level = nodes.last().map_or(0, |node: &Node| node.level());
    if level > previous_level + 1 {
      return Err(err_skipped_indentation_level(previous_level, level, line, indentation));
    }
    let node = Node::new(level, delimiter, name, content);
    nodes.push(node);
  }
  for mut node in nodes.drain(..).rev() {
    if stack.is_empty() {
      stack.push(node);
    } else {
      while stack.last().is_some_and(|last| last.level() > node.level()) {
        node.add_child(stack.pop().unwrap());
      }
      stack.push(node);
    }
  }
  let mut root = Node::root();
  while let Some(node) = stack.pop() {
    root.add_child(node);
  }
  Ok(root)
}
