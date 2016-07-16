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

const_list! {
    name = LIST_CONST,
    type = i32,
    values = [ 1,2,3,]
}

const_list! {
    name = EMPTY_LIST,
    type = i32,
    values = []
}

const_list! {
    name = STRING_LIST,
    type = &'static str,
    values = [ "hello", "world", ]
}

const_map! {
    name = MAP_CONST,
    ktype = i32,
    vtype = &'static str,
    values = {
        { 1, "foo" },
        { 2, "bar" },
    }
}

const_map! {
    name = EMPTY_MAP,
    ktype = i32,
    vtype = i32,
    values = {
    }
}