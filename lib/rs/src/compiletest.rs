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
        get_struct(key: i32 => 1,) -> DeeplyNested, DeeplyNested, GetStructExn => [],
        do_oneway(thing: i32 => 1,) -> oneway, oneway, oneway => [],
    ],
    parent = []
}

struct TestCtxt(i32);
use std::sync::{Arc, Mutex};
use self::shared_service::processor::SharedService;
impl SharedService for Arc<Mutex<TestCtxt>> {
    fn get_struct(&mut self, key: i32) -> common::DeeplyNested {
        let v = &mut self.lock().expect("lock failed").0;
        assert_eq!(key, *v);
        *v += key;
        common::DeeplyNested::default()
    }
    fn do_oneway(&mut self, _thing: i32) -> () { unimplemented!() }
}

service! {
     name = child_service,
     trait_name = ChildService,
     service_methods = [
         operation(
             one: String => 2,
             another: i32 => 3,
         ) -> Operation, Operation, OperationExn => [],
     ],
     parent = [ shared_service: SharedService ]
}

service! {
    name = service_with_exception,
    trait_name = ServiceWithException,
    service_methods = [
        operation() -> i32, ::std::result::Result<i32, OperationExn>, OperationExn => [bad Bad: Exception => 1,],
    ],
    parent = []
}

use self::service_with_exception::processor::{ServiceWithException, OperationExn};
impl ServiceWithException for Arc<Mutex<TestCtxt>> {
    fn operation(&mut self) -> Result<i32, OperationExn> {
        let _v = &mut self.lock().expect("lock failed").0;
        if unimplemented!() {
            Ok(*_v)
        } else {
            Err(Default::default())
        }
    }
}