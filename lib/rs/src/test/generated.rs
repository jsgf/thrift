strukt! {
    name = Simple,
    derive = [],
    reqfields = {},
    optfields = {
        key: String => 16, default = None,
    }
}

strukt! {
    name = Empty,
    derive = [],
    reqfields = {},
    optfields = {}
}

strukt! {
    name = Nested,
    derive = [],
    reqfields = {},
    optfields = {
        nested: Vec<Vec<Vec<Simple>>> => 32, default = None,
    }
}

strukt! {
    name = Recursive,
    derive = [],
    reqfields = {},
    optfields = {
        recurse: Vec<Recursive> => 0, default = None,
    }
}

strukt! {
     name = Many,
     derive = [],
     reqfields = {},
     optfields = {
         one: i32 => 3, default = None,
         two: String => 4, default = None,
         three: Vec<Simple> => 9, default = None,
     }
}

enom! {
    name = Operation,
    values = [
        Add = 1,
        Sub = 2,
        Clear = 3,
    ],
    default = Sub
}

konst! { const LIST_CONST: Vec<i32> = vec![ 1,2,3 ]}

konst! { const EMPTY_LIST: Vec<i32> = vec! [] }

konst! { const STRING_LIST: Vec<&'static str> = vec! [ "hello", "world" ]}

konst! { const MAP_CONST: ::std::collections::HashMap<i32, &'static str> = hashmap_literal! { 1 => "foo", 2 => "bar"}}

konst! { const EMPTY_MAP: ::std::collections::HashMap<i32, i32> = hashmap_literal! {}}

konst! { const SET_CONST: ::std::collections::HashSet<i32> = hashset_literal! [ 1, 2, 3, 3, 2] }

konst! { const EMPTY_SET: ::std::collections::HashSet<i32> = hashset_literal! [ ]}