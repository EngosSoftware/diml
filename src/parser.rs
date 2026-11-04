//! # Parser implementation

use crate::Node;
use crate::errors::Result;
use crate::segment::Segments;

/// Parses input.
///
/// # Errors
///
/// Returns an error when parsing invalid input.
///
/// # Panics
///
/// Unwraps are safe, no panics.
pub fn parse(input: &str) -> Result<Node<'_>> {
    fn append_child(stack: &mut Vec<(Node, usize)>) {
        let (child, _) = stack.pop().unwrap();
        stack.last_mut().unwrap().0.add_child(child);
    }
    let mut stack = vec![(Node::new(0, "", ""), 0)];
    for segment in Segments::new(input.char_indices().peekable()) {
        let segment = segment?;
        while stack
            .last()
            .is_some_and(|(_, level)| *level > segment.level())
        {
            append_child(&mut stack);
        }
        stack.push((
            Node::new(segment.line(), segment.name(input), segment.content(input)),
            segment.level() + 1,
        ));
    }
    while stack.len() > 1 {
        append_child(&mut stack);
    }
    Ok(stack.pop().unwrap().0)
}
