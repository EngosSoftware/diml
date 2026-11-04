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
pub fn parse(input: &str) -> Result<Node> {
    fn append_child(stack: &mut Vec<(Node, usize)>) {
        let (child, _) = stack.pop().unwrap();
        stack.last_mut().unwrap().0.add_child(child);
    }
    let mut stack = vec![(Node::new(0, String::new(), String::new()), 0)];
    for segment in Segments::new(input.char_indices().peekable()) {
        let segment = segment?;
        println!("name = {}", segment.name(input));
        println!("content = {}", segment.content(input));
        let segment_level = segment.level() + 1;
        while stack
            .last()
            .is_some_and(|(_, level)| *level >= segment_level)
        {
            append_child(&mut stack);
        }
        stack.push((
            Node::new(
                1,
                segment.name(input).to_string(),
                segment.content(input).to_string(),
            ),
            segment_level,
        ));
    }
    while stack.len() > 1 {
        append_child(&mut stack);
    }
    Ok(stack.pop().unwrap().0)
}
