#![allow(dead_code)]

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

service! {
    trait_name = SharedService,
    processor_name = SharedServiceProcessor,
    client_name = SharedServiceClient,
    service_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult SharedServiceGetStructExn = shared.get_struct(key: i32 => 1,) -> DeeplyNested => [],
    ],
    parent_methods = [],
    bounds = [S: SharedService,],
    fields = [shared: S,]
}

service! {
     trait_name = ChildService,
     processor_name = ChildServiceProcessor,
     client_name = ChildServiceClient,
     service_methods = [
         ChildServiceOperationArgs -> ChildServiceOperationResult ChildServiceOperationExn = child.operation(
             one: String => 2,
             another: i32 => 3,
         ) -> Operation => [],
     ],
     parent_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult SharedServiceGetStructExn = shared.get_struct(key: i32 => 1,) -> DeeplyNested => [],
     ],
     bounds = [S: SharedService, C: ChildService,],
     fields = [shared: S, child: C,]
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

service! {
    trait_name = ServiceWithException,
    processor_name = ServiceWithExceptionProcessor,
    client_name = ServiceWithExceptionClient,
    service_methods = [
        ServiceWithExceptionOperationArgs -> ServiceWithExceptionOperationResult ServiceWithExceptionOperationExn = this.operation() -> i32 => [bad Bad: Exception => 1,],
    ],
    parent_methods = [],
    bounds = [S: ServiceWithException,],
    fields = [this: S,]
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
