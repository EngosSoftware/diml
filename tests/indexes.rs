use diml::parse;

#[test]
fn _0001() {
  let input = r#".A
    .B
    .C
    .D
"#;
  let root = parse(input).unwrap();
  assert_eq!("A", root.first_with_name("A").unwrap().name());
  assert_eq!("B", root.first_with_name("A").unwrap().first_with_name("B").unwrap().name());
}

#[test]
fn _0002() {
  let input = r#".A
    .B
    .C
    .D
"#;
  let root = parse(input).unwrap();
  assert_eq!("A", root.last_with_name("A").unwrap().name());
  assert_eq!("D", root.last_with_name("A").unwrap().last_with_name("D").unwrap().name());
}
