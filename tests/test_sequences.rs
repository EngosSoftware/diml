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

const A: &str = ".\n";
const B: &str = ". b\n";
const C: &str = ".C\n";
const D: &str = ".D d\n";

fn eq(node1: &str, node2: &str) {
  let input = format!("{node1}{node2}");
  assert_eq!(input, format!("{}", parse(&input).unwrap()));
}

#[test]
fn sequences_should_work() {
  eq(A, B);
  eq(B, A);
  eq(A, C);
  eq(C, A);
  eq(A, D);
  eq(D, A);
  eq(B, C);
  eq(C, B);
  eq(B, D);
  eq(D, B);
  eq(C, D);
  eq(D, C);
}
