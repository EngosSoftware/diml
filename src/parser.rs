//! # Parser implementation

use crate::Node;
use crate::errors::Result;
use crate::segment::Segments;

/// Parses input.
///
/// # Returns
///
/// Document [Node].
///
/// All nodes parsed from input are children of the document [Node].
///
/// # Errors
///
/// Returns [`DimlError`](crate::DimlError) when parsing fails.
///
/// # Panics
///
/// No panics.
///
/// # Examples
///
/// Input contains the following text:
///
/// ```text
/// .A Ann
///   .B Bob
///   .C Clyde
///     .D Dave
/// ```
///
/// ```
/// use diml::parse;
///
/// # let input = ".A Ann\n  .B Bob\n  .C Clyde\n    .D Dave\n";
/// let doc = parse(input).unwrap();
///
/// let node_a = doc.children().next().unwrap();
/// assert_eq!(1, node_a.line());
/// assert_eq!("A", node_a.name());
/// assert_eq!("Ann", node_a.content().trim());
///
/// let mut children_a = node_a.children();
/// let node_b = children_a.next().unwrap();
/// assert_eq!(2, node_b.line());
/// assert_eq!("B", node_b.name());
/// assert_eq!("Bob", node_b.content().trim());
///
/// let node_c = children_a.next().unwrap();
/// assert_eq!(3, node_c.line());
/// assert_eq!("C", node_c.name());
/// assert_eq!("Clyde", node_c.content().trim());
///
/// let node_d = node_c.children().next().unwrap();
/// assert_eq!(4, node_d.line());
/// assert_eq!("D", node_d.name());
/// assert_eq!("Dave", node_d.content().trim());
/// ```
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
