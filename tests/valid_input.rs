use diml::parse;

#[test]
fn _0001() {
  let inputs = ["-\n", "-\r", "-\r\n", "- \n"];
  for input in inputs {
    let root = parse(input).unwrap();
    assert_eq!(1, root.children().count());
    let node = root.children().next().unwrap();
    assert_eq!('-', node.delimiter());
    assert_eq!("", node.name());
    assert_eq!(&input[1..], node.content());
    assert_eq!("", node.text());
  }
}

#[test]
fn _0002() {
  let input = "- node content\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('-', node.delimiter());
  assert_eq!("", node.name());
  assert_eq!(" node content\n", node.content());
  assert_eq!("node content", node.text());
}

#[test]
fn _0003() {
  let input = "ab\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("b", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
}

#[test]
fn _0004() {
  let input = "aa b\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("a", node.name());
  assert_eq!(" b\n", node.content());
  assert_eq!("b", node.text());
}

#[test]
fn _0005() {
  let input = "aa\nb\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("a", node.name());
  assert_eq!("\nb\n", node.content());
  assert_eq!("b", node.text());
}

#[test]
fn _0006() {
  let input = "aa\rb\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("a", node.name());
  assert_eq!("\rb\n", node.content());
  assert_eq!("b", node.text());
}

#[test]
fn _0007() {
  let input = "aa\r\nb\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("a", node.name());
  assert_eq!("\r\nb\n", node.content());
  assert_eq!("b", node.text());
}

#[test]
fn _0008() {
  let input = "aa\tb\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('a', node.delimiter());
  assert_eq!("a", node.name());
  assert_eq!("\tb\n", node.content());
  assert_eq!("b", node.text());
}

#[test]
fn _0009() {
  let input = ".A\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('.', node.delimiter());
  assert_eq!("A", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
}

#[test]
fn _0010() {
  let input = "😀_this_is-some_funny-name\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('😀', node.delimiter());
  assert_eq!("_this_is-some_funny-name", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
}

#[test]
fn _0011() {
  let input = ">z\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('>', node.delimiter());
  assert_eq!("z", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
}

#[test]
fn _0012() {
  let input = ".A1\n";
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  let node = root.children().next().unwrap();
  assert_eq!('.', node.delimiter());
  assert_eq!("A1", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
}

#[test]
fn _0013() {
  let input = ".A\n.k\n";
  let root = parse(input).unwrap();
  let mut children = root.children();
  let first = children.next().unwrap();
  assert_eq!('.', first.delimiter());
  assert_eq!("A", first.name());
  assert_eq!("\n", first.content());
  assert_eq!("", first.text());
  let second = children.next().unwrap();
  assert_eq!('.', second.delimiter());
  assert_eq!("k", second.name());
  assert_eq!("\n", second.content());
  assert_eq!("", second.text());
  assert!(children.next().is_none());
}

#[test]
fn _0014() {
  let input = "$x\n$_\n";
  let root = parse(input).unwrap();
  let mut children = root.children();
  let first = children.next().unwrap();
  assert_eq!('$', first.delimiter());
  assert_eq!("x", first.name());
  assert_eq!("\n", first.content());
  assert_eq!("", first.text());
  let second = children.next().unwrap();
  assert_eq!('$', second.delimiter());
  assert_eq!("_", second.name());
  assert_eq!("\n", second.content());
  assert_eq!("", second.text());
  assert!(children.next().is_none());
}

#[test]
fn _0015() {
  let input = "*n\n    *_\n";
  let root = parse(input).unwrap();
  let mut children = root.children();
  let first = children.next().unwrap();
  assert_eq!('*', first.delimiter());
  assert_eq!("n", first.name());
  assert_eq!("\n", first.content());
  assert_eq!("", first.text());
  let second = first.children().next().unwrap();
  assert_eq!('*', second.delimiter());
  assert_eq!("_", second.name());
  assert_eq!("\n", second.content());
  assert_eq!("", second.text());
  assert!(children.next().is_none());
}

#[test]
fn _0016() {
  let input = ".A\n$B\n";
  let root = parse(input).unwrap();
  let mut children = root.children();
  let first = children.next().unwrap();
  assert_eq!('.', first.delimiter());
  assert_eq!("A", first.name());
  assert_eq!("\n$B\n", first.content());
  assert_eq!("$B", first.text());
  assert!(children.next().is_none());
}

#[test]
fn _0017() {
  let input = ".A\r";
  let root = parse(input).unwrap();
  assert_eq!(input, root.document(4, ' '));
}

#[test]
fn _0018() {
  let input = ".A\r\n";
  let root = parse(input).unwrap();
  assert_eq!(input, root.document(4, ' '));
}

#[test]
fn _0019() {
  let input = r#".A
    .B
    .C
"#;
  let root = parse(input).unwrap();
  assert_eq!(input, root.document(4, ' '));
}

#[test]
fn _0020() {
  let input = r#".A

  some content

    .B
    .C
"#;
  let root = parse(input).unwrap();
  assert_eq!(input, root.document(4, ' '));
}

#[test]
fn _0021() {
  let input = r#"
.A a
  .B b
  
  .C c
"#;
  let root = parse(input.trim_start()).unwrap();
  let node_a = root.children().next().unwrap();
  assert_eq!('.', node_a.delimiter());
  assert_eq!("A", node_a.name());
  assert_eq!("a", node_a.text());
  assert_eq!(" a\n", node_a.content());
  let mut children = node_a.children();
  let node_b = children.next().unwrap();
  assert_eq!('.', node_b.delimiter());
  assert_eq!("B", node_b.name());
  assert_eq!("b", node_b.text());
  assert_eq!(" b\n  \n", node_b.content());
  let node_c = children.next().unwrap();
  assert_eq!('.', node_c.delimiter());
  assert_eq!("C", node_c.name());
  assert_eq!("c", node_c.text());
  assert_eq!(" c\n", node_c.content());
}
