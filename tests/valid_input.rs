use diml::parse;

#[test]
fn _0001() {
    let doc = parse(".\n").unwrap();
    assert_eq!(0, doc.line());
    assert_eq!("", doc.name());
    assert_eq!("", doc.content());
    assert_eq!(1, doc.children().len());
}

#[test]
fn _0002() {
    let doc = parse("-\n").unwrap();
    assert_eq!(0, doc.line());
    assert_eq!(1, doc.children().len());
    let node = doc.children().next().unwrap();
    assert_eq!(1, node.line());
    assert_eq!("", node.name());
    assert_eq!("\n", node.content());
    assert_eq!(0, node.children().len());
}

#[test]
fn _0003() {
    let inputs = ["-\n", "-\r", "-\r\n", "- \n"];
    for input in inputs {
        let doc = parse(input).unwrap();
        assert_eq!(1, doc.children().count());
        let node = doc.children().next().unwrap();
        assert_eq!("", node.name());
        assert_eq!(&input[1..], node.content());
    }
}

#[test]
fn _0004() {
    let input = "- node content\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("", node.name());
    assert_eq!(" node content\n", node.content());
}

#[test]
fn _0005() {
    let input = ".a\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0006() {
    let input = ".a\r\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\r\n", node.content());
}

#[test]
fn _0007() {
    let input = "ab\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("b", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0008() {
    let input = "aa b\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!(" b\n", node.content());
}

#[test]
fn _0009() {
    let input = "aa\nb\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\nb\n", node.content());
}

#[test]
fn _0010() {
    let input = "aa\rb\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\rb\n", node.content());
}

#[test]
fn _0011() {
    let input = "aa\r\nb\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\r\nb\n", node.content());
}

#[test]
fn _0012() {
    let input = "aa\tb\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("a", node.name());
    assert_eq!("\tb\n", node.content());
}

#[test]
fn _0013() {
    let input = ".A\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("A", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0014() {
    let input = "😀_this_is-some_funny-name\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("_this_is-some_funny-name", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0015() {
    let input = ">z\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("z", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0016() {
    let input = ".A1\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("A1", node.name());
    assert_eq!("\n", node.content());
}

#[test]
fn _0017() {
    let input = ".A\n.k\n";
    let doc = parse(input).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!("A", first.name());
    assert_eq!("\n", first.content());
    let second = children.next().unwrap();
    assert_eq!("k", second.name());
    assert_eq!("\n", second.content());
    assert!(children.next().is_none());
}

#[test]
fn _0018() {
    let input = "$x\n$_\n";
    let doc = parse(input).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!("x", first.name());
    assert_eq!("\n", first.content());
    let second = children.next().unwrap();
    assert_eq!("_", second.name());
    assert_eq!("\n", second.content());
    assert!(children.next().is_none());
}

#[test]
fn _0019() {
    let input = "*n\n    *_\n";
    let doc = parse(input).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!("n", first.name());
    assert_eq!("\n", first.content());
    let second = first.children().next().unwrap();
    assert_eq!("_", second.name());
    assert_eq!("\n", second.content());
    assert!(children.next().is_none());
}

#[test]
fn _0020() {
    let input = ".A\n$B\n";
    let doc = parse(input).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!("A", first.name());
    assert_eq!("\n$B\n", first.content());
    assert!(children.next().is_none());
}

#[test]
fn _0021() {
    let input = r".A

  some content

  .B
  .C
";
    let doc = parse(input).unwrap();
    let mut children = doc.children();
    let first = children.next().unwrap();
    assert_eq!("A", first.name());
    assert_eq!("\n\n  some content\n\n", first.content());
    let mut children = first.children();
    let second = children.next().unwrap();
    assert_eq!("B", second.name());
    assert_eq!("\n", second.content());
    let third = children.next().unwrap();
    assert_eq!("C", third.name());
    assert_eq!("\n", third.content());
}

#[test]
fn _0022() {
    let input = r"
.A a
  .B b
  
  .C c
";
    let doc = parse(input.trim_start()).unwrap();
    let node_a = doc.children().next().unwrap();
    assert_eq!("A", node_a.name());
    assert_eq!(" a\n", node_a.content());
    let mut children = node_a.children();
    let node_b = children.next().unwrap();
    assert_eq!("B", node_b.name());
    assert_eq!(" b\n  \n", node_b.content());
    let node_c = children.next().unwrap();
    assert_eq!("C", node_c.name());
    assert_eq!(" c\n", node_c.content());
}

#[test]
fn _0023() {
    // Control characters are allowed in the content.
    let input = ".A ab\u{7f}cd\n";
    let doc = parse(input).unwrap();
    assert_eq!(1, doc.children().count());
    let node = doc.children().next().unwrap();
    assert_eq!("A", node.name());
    assert_eq!(" ab\u{7f}cd\n", node.content());
}
