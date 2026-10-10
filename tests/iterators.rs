use diml::{Node, parse};

fn with_name(node: &Node, name: &str) -> Vec<String> {
  node.with_name(name).map(|node| node.name().to_string()).collect::<Vec<String>>()
}

fn with_names(node: &Node, names: &[&str]) -> Vec<String> {
  node.with_names(names).map(|node| node.name().to_string()).collect::<Vec<String>>()
}

fn excluding_name(node: &Node, name: &str) -> Vec<String> {
  node.excluding_name(name).map(|node| node.name().to_string()).collect::<Vec<String>>()
}

fn excluding_names(node: &Node, names: &[&str]) -> Vec<String> {
  node.excluding_names(names).map(|node| node.name().to_string()).collect::<Vec<String>>()
}

#[test]
fn _0001() {
  let input = r#".A
    .B
    .B
    .C
    .D
    .D
    .D
"#;
  let root = parse(input).unwrap();
  assert_eq!(1, root.children().count());
  assert_eq!(1, root.child_count());
  let node = root.children().next().unwrap();
  assert_eq!(6, node.children().count());
  assert_eq!(6, node.child_count());
  assert_eq!('.', node.delimiter().unwrap());
  assert_eq!("A", node.name());
  assert_eq!("\n", node.content());
  assert_eq!("", node.text());
  assert_eq!(vec!["B", "B"], with_name(node, "B"));
  assert_eq!(vec!["C"], with_name(node, "C"));
  assert_eq!(vec!["D", "D", "D"], with_name(node, "D"));
  assert!(with_name(node, "E").is_empty());
}

#[test]
fn _0002() {
  let input = r#".A
    .B
    .B
    .C
    .D
    .D
    .D
"#;
  let root = parse(input).unwrap();
  let node = root.children().next().unwrap();
  assert_eq!(vec!["C", "D", "D", "D"], excluding_name(node, "B"));
  assert_eq!(vec!["B", "B", "D", "D", "D"], excluding_name(node, "C"));
  assert_eq!(vec!["B", "B", "C"], excluding_name(node, "D"));
}

#[test]
fn _0003() {
  let input = r#".A
    .B
    .B
    .C
    .D
    .D
    .D
"#;
  let root = parse(input).unwrap();
  let node = root.children().next().unwrap();
  assert_eq!(vec!["B", "B"], with_names(node, &["B"]));
  assert_eq!(vec!["B", "B", "D", "D", "D"], with_names(node, &["B", "D"]));
  assert_eq!(vec!["B", "B", "C"], with_names(node, &["C", "B"]));
}

#[test]
fn _0004() {
  let input = r#".A
    .B
    .B
    .C
    .D
    .D
    .D
"#;
  let root = parse(input).unwrap();
  let node = root.children().next().unwrap();
  assert_eq!(vec!["C", "D", "D", "D"], excluding_names(node, &["B"]));
  assert_eq!(vec!["C"], excluding_names(node, &["B", "D"]));
  assert_eq!(vec!["D", "D", "D"], excluding_names(node, &["C", "B"]));
}
