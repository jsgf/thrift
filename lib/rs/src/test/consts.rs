use test::generated::*;
use std::collections::{HashMap, HashSet};

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),+) => {{
         let mut map = HashMap::new();
         $( map.insert($key, $val); )+
         map
    }};
    () => (HashMap::new())
}

macro_rules! hashset {
    ($($val:expr),+) => {{
        let mut set = HashSet::new();
        $( set.insert($val); )+
        set
    }};
    () => (HashSet::new())
}

#[test]
fn test_map() {
    let map = hashmap! { 1 => "foo", 2 => "bar" };
    assert_eq!(*MAP_CONST, map);
}

#[test]
fn test_map_empty() {
    let map = hashmap! {};
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

#[test]
fn test_set() {
    let set = hashset![1,2,3];
    assert_eq!(*SET_CONST, set);
}

#[test]
fn test_empty_set() {
    let set = hashset![];
    assert_eq!(*EMPTY_SET, set);
}