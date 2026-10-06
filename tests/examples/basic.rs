use super::EXAMPLE_BASIC;
use diml::parse;

#[test]
fn _0001() {
  let input = EXAMPLE_BASIC;
  let root = parse(input).unwrap();
  assert_eq!(6, root.children().count());
  let names = root.children().map(|node| node.name()).collect::<Vec<&str>>();
  assert_eq!(vec!["comment", "company", "domains", "tutorial", "author", "published"], names);
  let values = root.children().map(|node| node.text()).collect::<Vec<&str>>();
  assert_eq!(vec!["A sample DIML file", "Engos Software", "", "", "Dariusz Depta", "true"], values);
  let domain_names = root.first_with_name("domains").unwrap().children().map(|node| node.name()).collect::<Vec<&str>>();
  assert_eq!(vec!["", "", "", ""], domain_names);
  let domain_values = root.first_with_name("domains").unwrap().children().map(|node| node.text()).collect::<Vec<&str>>();
  assert_eq!(vec!["business analysts", "software developers", "data engineers", "devops"], domain_values);
  assert_eq!(
    "Brilliant!",
    root
      .first_with_name("tutorial")
      .unwrap()
      .first_with_name("diml")
      .unwrap()
      .first_with_name("type")
      .unwrap()
      .text()
  );
}
