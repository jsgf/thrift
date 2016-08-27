/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate thrift;
extern crate bufstream;
extern crate getopts;

use getopts::Options;
use std::env;

use std::net::TcpListener;

use thrift::server::SimpleServer;
use thrift::server::ThreadedServer;
use thrift::server::Server;
use thrift::protocol::binary_protocol::BinaryProtocol;

mod thrift_test;
mod small_test;

use thrift_test::thrift_test::processor::{self, ThriftTest};
use thrift_test::common::*;
use thrift_test::common::Numberz::*;

macro_rules! map {
    () => { Map::new() };
    ( $($key:expr => $val:expr ),* $(,)* ) => {
        {
            let mut map = Map::new();
            $( map.insert($key.into(), $val.into()); )+
            map
        }
    }
}

#[derive(Debug, Clone)]
struct TestService;

impl TestService {
    fn new() -> Self { TestService }
}

macro_rules! loopback {
    ($name:ident => $ty:ty) => {
        fn $name(&mut self, v: $ty) -> $ty {
            println!("{}: {:?}", stringify!($name), v);
            v
        }
    }
}

impl ThriftTest for TestService {
    fn testVoid(&mut self) {
        println!("testVoid");
    }

    loopback!(testString => String);
    loopback!(testBool => bool);
    loopback!(testByte => i8);
    loopback!(testI32 => i32);
    loopback!(testI64 => i64);
    loopback!(testDouble => f64);
    loopback!(testBinary => Vec<u8>);
    loopback!(testStruct => Xtruct);
    loopback!(testNest => Xtruct2);
    loopback!(testMap => Map<i32, i32>);
    loopback!(testStringMap => Map<String, String>);
    loopback!(testSet => Set<i32>);
    loopback!(testList => Vec<i32>);
    loopback!(testEnum => Numberz);
    loopback!(testTypedef => UserId);

    fn testMapMap(&mut self, hello: i32) -> Map<i32, Map<i32, i32>> {
        println!("testMapMap(hello: {})", hello);

        let pos: Map<_,_> = (1..5).into_iter().map(|i| (i, i)).collect();
        let neg: Map<_,_> = (1..5).into_iter().map(|i| (-i, -i)).collect();

        map!{ -4 => neg, 4 => pos }
    }

    fn testInsanity(&mut self, argument: Insanity) -> Map<UserId, Map<Numberz, Insanity>> {
        print!("testInsanity(argument: {:?})", argument);

        let first_map = map!{ TWO => argument.clone(), THREE => argument.clone() };
        let second_map = map! { SIX => Insanity::default() };
        let insane = map! { 1 => first_map, 2 => second_map };

        println!(" -> {:?}", insane);

        insane
    }

    fn testMulti(&mut self, arg0: i8, arg1: i32, arg2: i64, arg3: Map<i16, String>, arg4: Numberz, arg5: UserId) -> Xtruct {
        println!("testMulti(arg0: {}, arg1: {}, arg2: {}, arg3:{:?}, arg4: {:?}, arg5: {})",
                 arg0, arg1, arg2, arg3, arg4, arg5);
        Xtruct {
            string_thing: Some("Hello2".into()),
            byte_thing: Some(arg0),
            i32_thing: Some(arg1),
            i64_thing: Some(arg2),
        }
    }

    fn testException(&mut self, arg: String) -> Result<(), processor::TestExceptionExn> {
        println!("testException(s: {:?})", arg);

        match arg.as_ref() {
            "Xception" => Err(processor::TestExceptionExn::Err1(Xception {
                                    error_code: Some(1001), message: Some(arg.clone())
                                })),
            "TException" => Err(processor::TestExceptionExn::Unknown),
            _ => Ok(()),
        }
    }

    fn testMultiException(&mut self, arg0: String, arg1: String) -> Result<Xtruct, processor::TestMultiExceptionExn> {
        println!("testMultiException(arg0: {}, arg1: {})", arg0, arg1);

        match arg0.as_ref() {
            "Xception" => Err(processor::TestMultiExceptionExn::Err1(Xception {
                error_code: Some(1001), message: Some("This is an Xception".into())
            })),
            "Xception2" => Err(processor::TestMultiExceptionExn::Err2(Xception2 {
                error_code: Some(2002),
                struct_thing: Some(Xtruct { string_thing: Some("This is an Xception2".into()), ..Default::default() })
            })),
            _ => Ok(Xtruct { string_thing: Some(arg1), ..Default::default() })
        }
    }

    fn testOneway(&mut self, v: i32) {
        println!("testOneway {}", v)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //let program = args[0].clone();

    let mut opts = Options::new();

    opts.optopt("H", "host", "host", "HOST");
    opts.optopt("P", "port", "port number", "PORT");
    opts.optopt("", "protocol", "Thrift protocol", "");
    opts.optopt("", "transport", "Thrift transport", "");
    opts.optopt("", "server", "server type", "");

    let mut port = 9090;
    let mut host = String::from("localhost");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("Failed to parse args: {:?}", e),
    };

    if let Some(h) = matches.opt_str("host") {
        host = h;
    }
    if let Some(p) = matches.opt_str("port") {
        port = p.parse().expect("port number");
    }
    let listener = TcpListener::bind((host.as_ref(), port)).expect("bind failed");
    let service = TestService::new();

    let mut server: Box<Server> =
        match matches.opt_str("server").as_ref().map(|s| s.as_str()) {
            Some("threaded") => {
                println!("threaded server");
                let mut s = ThreadedServer::new(listener, BinaryProtocol::new, 5);
                processor::register(&mut s, &service);
                Box::new(s)
            },
            Some("simple") | _ => {
                println!("simple server");
                let mut s = SimpleServer::new(listener, BinaryProtocol::new);
                processor::register(&mut s, &service);
                Box::new(s)
            },
        };

    server.serve();
}