strukt! {
    name = Simple,
    fields = {
        key: String => 16,
    }
}

strukt! {
    name = Empty,
    fields = {}
}

strukt! {
    name = Nested,
    fields = {
        nested: Vec<Vec<Vec<Simple>>> => 32,
    }
}

strukt! {
    name = Recursive,
    fields = {
        recurse: Vec<Recursive> => 0,
    }
}

strukt! {
     name = Many,
     fields = {
         one: i32 => 3,
         two: String => 4,
         three: Vec<Simple> => 9,
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

konst! { const MAP_CONST: ::std::collections::HashMap<i32, &'static str> = map_literal! { 1 => "foo", 2 => "bar"}}

konst! { const EMPTY_MAP: ::std::collections::HashMap<i32, i32> = map_literal! {}}

konst! { const SET_CONST: ::std::collections::HashSet<i32> = set_literal! [ 1, 2, 3, 3, 2] }

konst! { const EMPTY_SET: ::std::collections::HashSet<i32> = set_literal! [ ]}