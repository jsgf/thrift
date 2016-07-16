use test::generated::*;
use std::collections::HashMap;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

#[test]
fn test_map() {
    let map = hashmap! { 1 => "foo", 2 => "bar" };
    assert_eq!(*MAP_CONST, map);
}

#[test]
fn test_map_empty() {
    let map = HashMap::new();
    assert_eq!(*EMPTY_MAP, map);
}

#[test]
fn test_list() {
    let l = vec![1,2,3];
    assert_eq!(*LIST_CONST, l);
}

#[test]
fn test_str_list() {
    let l = vec!["hello", "world"];
    assert_eq!(*STRING_LIST, l);
}

#[test]
fn test_empty_list() {
    let l = vec![];
    assert_eq!(*EMPTY_LIST, l);
}