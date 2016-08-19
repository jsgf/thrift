#![allow(dead_code)]

mod common {
    strukt! {
        name = Simple,
        derive = [ Eq, PartialEq, Debug, Hash, ],
        reqfields = {},
        optfields = {
            key: String => 16, default = Default::default(),
        }
    }

    strukt! {
        name = DeeplyNested,
        derive = [ Eq, PartialEq, Debug, ],
        reqfields = {},
        optfields = {
            nested: ::std::collections::HashSet<Vec<Vec<Vec<Vec<i32>>>>> => 6, default = Default::default(),
        }
    }

    strukt! {
        name = ReferencesOther,
        derive = [ Eq, PartialEq, Debug, ],
        reqfields = {},
        optfields = {
            other: DeeplyNested => 2, default = Default::default(),
            another: Simple => 3, default = Default::default(),
            map: ::std::collections::HashMap<i32, Vec<String>> => 4, default = Default::default(),
        }
    }

    enom! {
        name = Operation,
        values = [Add = 1, Sub = 2, Mul = 3, Div = 4,],
        default = Add
    }


    strukt! {
        name = Exception,
        derive = [ Eq, PartialEq, Debug, Hash, ],
        reqfields = {},
        optfields = {
            name: String => 0, default = Default::default(),
            message: String => 1, default = Default::default(),
        }
    }

    union! {
        name = TestUnion,
        derive = [Debug, PartialEq, ],
        default = TestUnion::Unknown,
        fields = {
            StringField: String => 1,
            I32Field: i32 => 2,
            StructList: Vec<Simple> => 4,
            OtherI32Field: i32 => 5,
            I32Set: ::std::collections::HashSet<i32> => 7,
            I32Map: ::std::collections::HashMap<i32, i32> => 8,
        }
    }

    union! {
        name = TestUnionDefl,
        derive = [Debug, PartialEq, ],
        default = TestUnionDefl::StringField("foo".into()),
        fields = {
            StringField: String => 1,
            I32Field: i32 => 2,
            StructList: Vec<Simple> => 4,
            OtherI32Field: i32 => 5,
            I32Set: ::std::collections::HashSet<i32> => 7,
            I32Map: ::std::collections::HashMap<i32, i32> => 8,
        }
    }
}

service! {
    name = shared_service,
    trait_name = SharedService,
    service_methods = [
        GetStructArgs -> GetStructResult GetStructExn = shared.get_struct(key: i32 => 1,) -> DeeplyNested, DeeplyNested => [],
        OnewayArgs -> OnewayResult OnewayExn = shared.oneway(thing: i32 => 1,) -> (), () => [],
    ],
    parent = [],
    bounds = [S: SharedService,],
    fields = [shared: S,]
}

service! {
     name = child_service,
     trait_name = ChildService,
     service_methods = [
         OperationArgs -> OperationResult OperationExn = child.operation(
             one: String => 2,
             another: i32 => 3,
         ) -> Operation, Operation => [],
     ],
     parent = [ shared_service: SharedService ],
     bounds = [S: SharedService, C: ChildService,],
     fields = [shared: S, child: C,]
}

service! {
    name = service_with_exception,
    trait_name = ServiceWithException,
    service_methods = [
        OperationArgs -> OperationResult OperationExn = this.operation() -> i32, ::std::result::Result<i32, OperationExn> => [bad Bad: Exception => 1,],
    ],
    parent = [],
    bounds = [S: ServiceWithException,],
    fields = [this: S,]
}
