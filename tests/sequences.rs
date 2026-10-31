//! Tests for node sequences, to assure, the proper node name
//! and node content is parsed.
//!
//! Node values:
//! - 00: no name, no content
//! - 01: no name, some content
//! - 10: some name, no content
//! - 11: some name, some content
//!
//! All possible sequences of nodes that cover all possible node values
//! are covered by 12 tests, that logically form a bidirectional graph
//! of four nodes.
//!
//! When node values are named as follows:
//! - a => 00
//! - b => 01
//! - c => 10
//! - d => 11
//!
//! then the following sequences of nodes are possible:
//! - ab
//! - ba
//! - ac
//! - ca
//! - ad
//! - da
//! - bc
//! - cb
//! - bd
//! - db
//! - cd
//! - dc

use diml::parse;

/// No name, no content.
const A: &str = ".\n";

/// No name, some content.
const B: &str = ". b\n";

/// Some name, no content.
const C: &str = ".C\n";

/// Some name, some content.
const D: &str = ".D d\n";

fn eq(node1: &str, node2: &str, name1: &str, content1: &str, name2: &str, content2: &str) {
    let doc = parse(&format!("{node1}{node2}")).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!(Some('.'), first.delimiter());
    assert_eq!(name1, first.name());
    assert_eq!(content1, first.content());
    let second = children.next().unwrap();
    assert_eq!(Some('.'), second.delimiter());
    assert_eq!(name2, second.name());
    assert_eq!(content2, second.content());
}

#[test]
fn sequences_should_work() {
    eq(A, B, "", "\n", "", " b\n");
    eq(B, A, "", " b\n", "", "\n");
    eq(A, C, "", "\n", "C", "\n");
    eq(C, A, "C", "\n", "", "\n");
    eq(A, D, "", "\n", "D", " d\n");
    eq(D, A, "D", " d\n", "", "\n");
    eq(B, C, "", " b\n", "C", "\n");
    eq(C, B, "C", "\n", "", " b\n");
    eq(B, D, "", " b\n", "D", " d\n");
    eq(D, B, "D", " d\n", "", " b\n");
    eq(C, D, "C", "\n", "D", " d\n");
    eq(D, C, "D", " d\n", "C", "\n");
}
