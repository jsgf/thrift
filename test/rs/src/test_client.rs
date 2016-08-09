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

use std::net::TcpStream;
use bufstream::BufStream;
use thrift::protocol::binary_protocol::BinaryProtocol;

mod thrift_test;
mod small_test;

macro_rules! basetype_identity {
    ( $client:expr , $func:ident, $val:expr ) => {
        match $client.$func($val) {
            Err(e) => panic!("failed on {} {:?}: {:?}", stringify!($func), $val, e),
            Ok(res) if res == $val => println!("OK {} {:?}", stringify!($func), $val),
            Ok(bad) => println!("failed on {} {:?} got {:?}", stringify!($func), $val, bad),
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

    let mut client = thrift_test::ThriftTestClient::new(BinaryProtocol, stream);

    match client.testVoid() {
        Err(e) => panic!("testVoid failed: {:?}", e),
        Ok(_) => println!("testVoid OK"),
    };

    match client.testString("Test".into()) {
        Err(e) => panic!("testString failed: {:?}", e),
        Ok(thrift_test::ThriftTestTestStringResult { success: Some(ref s) }) if s == "Test" =>
            println!("testString OK {}", s),
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

        match client.testString(s.into()) {
            Err(e) => panic!("testString long failed {:?}", e),
            Ok(thrift_test::ThriftTestTestStringResult { success: Some(ref rs) }) if rs == s =>
                println!("testString long OK"),
            Ok(bad) => panic!("testString long bad result {:?}", bad),
        }
    }

    {
        let s = 
          "quote: \" backslash:
           forwardslash-escaped: \\/ 
           backspace: \010 formfeed: \014 newline: \012 return: \013 tab: \011
           now-all-of-them-together: \"\\\\/\010\012\013\011
           now-a-bunch-of-junk: !@#$%&()(&%$#{}{}<><><
           char-to-test-json-parsing: ]] \"]] \\\" }}}{ [[[ ";

        match client.testString(s.into()) {
            Err(e) => panic!("testString json failed {:?}", e),
            Ok(thrift_test::ThriftTestTestStringResult { success: Some(ref rs) }) if rs == s =>
                println!("testString json OK"),
            Ok(bad) => panic!("testString json bad result {:?}", bad),
        }
    }

    basetype_identity!(client, testBool, true);
    basetype_identity!(client, testBool, false);
}