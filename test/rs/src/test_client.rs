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

use std::collections::{BTreeMap, BTreeSet};

use std::net::TcpStream;
use bufstream::BufStream;
use thrift::protocol::binary_protocol::BinaryProtocol;

mod thrift_test;
mod small_test;

use thrift_test::*;

macro_rules! map {
    () => { BTreeMap::new() };
    ( $($key:expr => $val:expr ),* $(,)* ) => {
        {
            let mut map = BTreeMap::new();
            $( map.insert($key.into(), $val.into()); )+
            map
        }
    }
}

macro_rules! loopback {
    ( $client:expr, $method:ident, $val:expr ) => {
        let name = stringify!($method);
        let val = $val;
        print!("{}({:?}) => ", name, val);
        match $client.$method(val.clone()) {
            Ok(Some(ref v)) if v == &val => println!("{} OK", name),
            Ok(bad) => panic!("{} failed bad {:?}", name, bad),
            Err(err) => panic!("{} failed err {:?}", name, err),
        }
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

    let stream = BufStream::new(TcpStream::connect((host.as_ref(), port)).unwrap());

    let mut client = ThriftTestClient::new(BinaryProtocol, stream);

    match client.testVoid() {
        Err(e) => panic!("testVoid failed: {:?}", e),
        Ok(res) => println!("testVoid OK res {:?}", res),
    };

    match client.testString("Test".into()) {
        Err(e) => panic!("testString failed: {:?}", e),
        Ok(Some(ref s)) if s == "Test" => println!("testString OK {}", s),
        Ok(bad) => panic!("testString bad result {:?}", bad)
    }

    {
        let s = 
          "}{Afrikaans, Alemannisch, Aragonés, العربية, مصرى, 
          Asturianu, Aymar aru, Azərbaycan, Башҡорт, Boarisch, Žemaitėška, 
          Беларуская, Беларуская (тарашкевіца), Български, Bamanankan, 
          বাংলা, Brezhoneg, Bosanski, Català, Mìng-dĕ̤ng-ngṳ̄, Нохчийн, 
          Cebuano, ᏣᎳᎩ, Česky, Словѣ́ньскъ / ⰔⰎⰑⰂⰡⰐⰠⰔⰍⰟ, Чӑвашла, Cymraeg, 
          Dansk, Zazaki, ދިވެހިބަސް, Ελληνικά, Emiliàn e rumagnòl, English, 
          Esperanto, Español, Eesti, Euskara, فارسی, Suomi, Võro, Føroyskt, 
          Français, Arpetan, Furlan, Frysk, Gaeilge, 贛語, Gàidhlig, Galego, 
          Avañe'ẽ, ગુજરાતી, Gaelg, עברית, हिन्दी, Fiji Hindi, Hrvatski, 
          Kreyòl ayisyen, Magyar, Հայերեն, Interlingua, Bahasa Indonesia, 
          Ilokano, Ido, Íslenska, Italiano, 日本語, Lojban, Basa Jawa, 
          ქართული, Kongo, Kalaallisut, ಕನ್ನಡ, 한국어, Къарачай-Малкъар, 
          Ripoarisch, Kurdî, Коми, Kernewek, Кыргызча, Latina, Ladino, 
          Lëtzebuergesch, Limburgs, Lingála, ລາວ, Lietuvių, Latviešu, Basa 
          Banyumasan, Malagasy, Македонски, മലയാളം, मराठी, مازِرونی, Bahasa 
          Melayu, Nnapulitano, Nedersaksisch, नेपाल भाषा, Nederlands, ‪
          Norsk (nynorsk)‬, ‪Norsk (bokmål)‬, Nouormand, Diné bizaad, 
          Occitan, Иронау, Papiamentu, Deitsch, Polski, پنجابی, پښتو, 
          Norfuk / Pitkern, Português, Runa Simi, Rumantsch, Romani, Română, 
          Русский, Саха тыла, Sardu, Sicilianu, Scots, Sámegiella, Simple 
          English, Slovenčina, Slovenščina, Српски / Srpski, Seeltersk, 
          Svenska, Kiswahili, தமிழ், తెలుగు, Тоҷикӣ, ไทย, Türkmençe, Tagalog, 
          Türkçe, Татарча/Tatarça, Українська, اردو, Tiếng Việt, Volapük, 
          Walon, Winaray, 吴语, isiXhosa, ייִדיש, Yorùbá, Zeêuws, 中文, 
          Bân-lâm-gú, 粵語";

        loopback!(client, testString, String::from(s));
    }

    {
        let s = 
          "quote: \" backslash:
           forwardslash-escaped: \\/ 
           backspace: \010 formfeed: \014 newline: \012 return: \013 tab: \011
           now-all-of-them-together: \"\\\\/\010\012\013\011
           now-a-bunch-of-junk: !@#$%&()(&%$#{}{}<><><
           char-to-test-json-parsing: ]] \"]] \\\" }}}{ [[[ ";

        loopback! (client, testString, String::from(s));
    }

    // bool
    loopback!(client, testBool, true);
    loopback!(client, testBool, false);

    // i8
    loopback!(client, testByte, 0);
    loopback!(client, testByte, -1);
    loopback!(client, testByte, 42);
    loopback!(client, testByte, -42);
    loopback!(client, testByte, 127);
    loopback!(client, testByte, -128);

    // i32
    loopback!(client, testI32, 0);
    loopback!(client, testI32, -1);
    loopback!(client, testI32, 190000013);
    loopback!(client, testI32, -190000013);
    loopback!(client, testI32, std::i32::MAX);
    loopback!(client, testI32, std::i32::MIN);

    // i64
    loopback!(client, testI64, 0);
    loopback!(client, testI64, -1);
    loopback!(client, testI64, 7000000000000000123_i64);
    loopback!(client, testI64, -7000000000000000123_i64);
    loopback!(client, testI64, 2_i64.pow(32));
    loopback!(client, testI64, -2_i64.pow(32));
    loopback!(client, testI64, 2_i64.pow(32) + 1);
    loopback!(client, testI64, -2_i64.pow(32) - 1);
    loopback!(client, testI64, std::i64::MAX);
    loopback!(client, testI64, std::i64::MIN);

    // f64
    // Comparing double values with plain equality because Thrift handles full precision of double
    loopback!(client, testDouble, 0.0);
    loopback!(client, testDouble, -1.0);
    loopback!(client, testDouble, -5.2098523);
    loopback!(client, testDouble, -0.000341012439638598279);
    loopback!(client, testDouble, 2_f64.powi(32));
    loopback!(client, testDouble, 2_f64.powi(32) + 1_f64);
    loopback!(client, testDouble, 2_f64.powi(53) - 1_f64);
    loopback!(client, testDouble, -2_f64.powi(32));
    loopback!(client, testDouble, -2_f64.powi(32) - 1_f64);
    loopback!(client, testDouble, -2_f64.powi(53) + 1_f64);

    {
        let expected = 10_f64.powi(307);
        match client.testDouble(expected) {
            Ok(Some(actual)) if expected - actual <= 10_f64.powi(292) => println!("testDouble OK expected {} actual {}", expected, actual),
            Ok(bad) => panic!("testDouble failed expected {} got {:?}", expected, bad),
            Err(err) => panic!("testDouble failed err {:?}", err),
        }
    }

    {
        let expected = 10_f64.powi(-292);
        match client.testDouble(expected) {
            Ok(Some(actual)) if expected - actual <= 10_f64.powi(-307) => println!("testDouble OK expected {} actual {}", expected, actual),
            Ok(bad) => panic!("testDouble failed expected {} got {:?}", expected, bad),
            Err(err) => panic!("testDouble failed err {:?}", err),
        }
    }

    match client.testBinary(vec![]) {
        Ok(Some(ref v)) if v.len() == 0 => println!("testBinary empty OK"),
        Ok(bad) => panic!("testBinary failed bad={:?}", bad),
        Err(err) => panic!("testBinary failed err {:?}", err),
    }

    {
        let data: Vec<u8> = [
            -128_i8, -127, -126, -125, -124, -123, -122, -121, -120, -119, -118, -117, -116, -115, -114,
           -113, -112, -111, -110, -109, -108, -107, -106, -105, -104, -103, -102, -101, -100, -99,
           -98,  -97,  -96,  -95,  -94,  -93,  -92,  -91,  -90,  -89,  -88,  -87,  -86,  -85,  -84,
           -83,  -82,  -81,  -80,  -79,  -78,  -77,  -76,  -75,  -74,  -73,  -72,  -71,  -70,  -69,
           -68,  -67,  -66,  -65,  -64,  -63,  -62,  -61,  -60,  -59,  -58,  -57,  -56,  -55,  -54,
           -53,  -52,  -51,  -50,  -49,  -48,  -47,  -46,  -45,  -44,  -43,  -42,  -41,  -40,  -39,
           -38,  -37,  -36,  -35,  -34,  -33,  -32,  -31,  -30,  -29,  -28,  -27,  -26,  -25,  -24,
           -23,  -22,  -21,  -20,  -19,  -18,  -17,  -16,  -15,  -14,  -13,  -12,  -11,  -10,  -9,
           -8,   -7,   -6,   -5,   -4,   -3,   -2,   -1,   0,    1,    2,    3,    4,    5,    6,
           7,    8,    9,    10,   11,   12,   13,   14,   15,   16,   17,   18,   19,   20,   21,
           22,   23,   24,   25,   26,   27,   28,   29,   30,   31,   32,   33,   34,   35,   36,
           37,   38,   39,   40,   41,   42,   43,   44,   45,   46,   47,   48,   49,   50,   51,
           52,   53,   54,   55,   56,   57,   58,   59,   60,   61,   62,   63,   64,   65,   66,
           67,   68,   69,   70,   71,   72,   73,   74,   75,   76,   77,   78,   79,   80,   81,
           82,   83,   84,   85,   86,   87,   88,   89,   90,   91,   92,   93,   94,   95,   96,
           97,   98,   99,   100,  101,  102,  103,  104,  105,  106,  107,  108,  109,  110,  111,
           112,  113,  114,  115,  116,  117,  118,  119,  120,  121,  122,  123,  124,  125,  126,
           127
        ].iter().map(|v| *v as u8).collect();
        assert_eq!(data.len(), 256);
        loopback!(client, testBinary, data);
    }

    {
        let out = Xtruct {
            string_thing: Some("Zero".into()),
            byte_thing: Some(1),
            i32_thing: Some(-3),
            i64_thing: Some(-5),
        };
        loopback!(client, testStruct, out);
    }

    {
        let out = Xtruct2 {
            struct_thing: Some(Xtruct {
                string_thing: Some("Zero".into()),
                byte_thing: Some(1),
                i32_thing: Some(-3),
                i64_thing: Some(-5),
            }),
            byte_thing: Some(1),
            i32_thing: Some(5),
        };
        loopback!(client, testNest, out);
    }

    {
        let mut map = BTreeMap::new();
        for i in 0..5 {
            let _ = map.insert(i, i - 10);
        }
        loopback!(client, testMap, map);
    }

    {
        let map = map! { "b" => "b2", "a" => "abbbaababab", "blonkzonk" => "zooblezonk" };
        loopback!(client, testStringMap, map);
    }

    {
        let mut set = BTreeSet::new();
        for i in -3..10 {
            let _ = set.insert(i);
        }
        loopback!(client, testSet, set);
    }

    loopback!(client, testList, vec![]);
    loopback!(client, testList, vec![-3,-2,1,4,5]);

    {
        use thrift_test::Numberz::*;
        loopback!(client, testEnum, ONE);
        loopback!(client, testEnum, TWO);
        loopback!(client, testEnum, THREE);
        loopback!(client, testEnum, FIVE);
        loopback!(client, testEnum, SIX);
        loopback!(client, testEnum, EIGHT);
    }

    loopback!(client, testTypedef, 309858235082523_i64);

    match client.testMapMap(1) {
        Ok(Some(v)) => {
            // XXX TODO: check proper return
            println!("testMapMap(1) returned {:?}", v);
        },
        Ok(bad) => panic!("testMapMap returned bad {:?}", bad),
        Err(err) => panic!("testMapMap returned err {:?}", err),
    }

    {
        use thrift_test::Numberz::*;

        let insane = Insanity {
            user_map: Some(map! { FIVE => 5, EIGHT => 8 }),
            xtructs: Some(vec! [
                Xtruct {
                    string_thing: Some("Goodbye4".into()),
                    byte_thing: Some(4),
                    i32_thing: Some(4),
                    i64_thing: Some(4),
                },
                Xtruct {
                    string_thing: Some("Hello2".into()),
                    byte_thing: Some(2),
                    i32_thing: Some(2),
                    i64_thing: Some(4),
                }
            ]),
        };
        match client.testInsanity(insane.clone()) {
            Ok(Some(ref v)) => {
                // XXX TODO: check proper return
                println!("testInsanity({:?}) -> {:?}", insane, v)
            },
            Ok(bad) => panic!("testInsanity({:?}) returned bad {:?}", insane, bad),
            Err(err) => panic!("testInsanity failed err {:?}", err),
        }
    }

    match client.testMulti(42, 4242, 424242, map! { 1_i16 => "blah", 2_i16 => "thing" }, Numberz::EIGHT, 24) {
        Ok(Some(ref res)) if res == &Xtruct { string_thing: Some("Hello2".into()), byte_thing: Some(42), i32_thing: Some(4242), i64_thing: Some(424242) } =>
            println!("testMulti OK got res {:?}", res),
        Ok(bad) => panic!("testMulti failed bad {:?}", bad),
        Err(err) => panic!("testMulti failed err {:?}", err),
    }

    match client.testException("Xception".into()) {
        Ok(Ok(bad)) => panic!("testException didn't except bad {:?}", bad),
        Ok(Err(ThriftTestTestExceptionResult { success: None, err1: Some(ref exn) }))
            if exn == &Xception { error_code: Some(1001), message: Some("Xception".into()) } =>
                println!("testException got {:?}", exn),
        Ok(Err(bad)) => panic!("testException didn't except bad {:?}", bad),
        Err(err) => panic!("testException got err {:?}", err),
    }

    // XXX FIXME void result returns None not ()
    match client.testException("success".into()) {
        Ok(Ok(())) => println!("testException(success) OK"),
        Ok(bad) => panic!("testException(success) failed bad {:?}", bad),
        Err(err) => panic!("testException(success) failed err {:?}", err),
    }
}