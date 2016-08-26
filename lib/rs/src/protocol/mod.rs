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
use std::{str, fmt};
use std::error::Error as StdError;

use Result;

pub mod binary_protocol;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Protocol version mismatch
    BadVersion,
    /// Sender violated the protocol, for instance, sent an unknown enum value
    ProtocolViolation,
    /// Sender sent a user exception the receiver wasn't expecting any exceptions
    UserException,
    /// Received string cannot be converted to a UTF8 string
    InvalidUtf8(str::Utf8Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Thrift Protocol Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
             Error::InvalidUtf8(ref e) => Some(e),
             _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::InvalidUtf8(e)
    }
}

pub trait ProtocolFactory {
    type Protocol: Protocol;

    fn new_protocol(&self) -> Self::Protocol;
}

impl<F, P: Protocol> ProtocolFactory for F where F: Fn() -> P {
    type Protocol = P;

    fn new_protocol(&self) -> P {
        (*self)()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Stop = 0x00,
    Void = 0x01,
    Bool = 0x02,
    I8 = 0x03,
    Double = 0x04,
    I16 = 0x06,
    I32 = 0x08,
    I64 = 0x0a,
    String = 0x0b,
    Struct = 0x0c,
    Map = 0x0d,
    Set = 0x0e,
    List = 0x0f
}

impl Type {
    pub fn from_num(num: u64) -> Option<Type> {
        match num {
            0x00 => Some(Type::Stop),
            0x01 => Some(Type::Void),
            0x02 => Some(Type::Bool),
            0x03 => Some(Type::I8),
            0x04 => Some(Type::Double),
            0x06 => Some(Type::I16),
            0x08 => Some(Type::I32),
            0x0a => Some(Type::I64),
            0x0b => Some(Type::String),
            0x0c => Some(Type::Struct),
            0x0d => Some(Type::Map),
            0x0e => Some(Type::Set),
            0x0f => Some(Type::List),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MessageType {
    Call = 0x01,
    Reply = 0x02,
    Exception = 0x03,
    Oneway = 0x04,
}

impl MessageType {
    pub fn from_num(num: u64) -> Option<MessageType> {
        match num {
            0x01 => Some(MessageType::Call),
            0x02 => Some(MessageType::Reply),
            0x03 => Some(MessageType::Exception),
            0x04 => Some(MessageType::Oneway),
            _ => None,
        }
    }
}

pub trait ThriftTyped {
    fn typ() -> Type;
}

pub trait Encode: ThriftTyped {
    fn encode<P>(&self, &mut P) -> Result<()>
        where P: Protocol;
}

pub trait Decode: ThriftTyped + Default {
    fn decode<P>(&mut P) -> Result<Self>
        where P: Protocol;
}

pub trait Protocol {
    fn write_message_begin(
        &mut self,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()>;
    fn write_message_end(&mut self) -> Result<()>;

    fn write_struct_begin(&mut self, name: &str) -> Result<()>;
    fn write_struct_end(&mut self) -> Result<()>;

    fn write_field_begin(
        &mut self,
        name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()>;
    fn write_field_end(&mut self) -> Result<()>;
    fn write_field_stop(&mut self) -> Result<()>;

    fn write_map_begin(
        &mut self,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()>;
    fn write_map_end(&mut self) -> Result<()>;

    fn write_list_begin(&mut self, elem_type: Type, size: usize) -> Result<()>;
    fn write_list_end(&mut self) -> Result<()>;

    fn write_set_begin(&mut self, elem_type: Type, size: usize) -> Result<()>;
    fn write_set_end(&mut self) -> Result<()>;

    fn write_bool(&mut self, value: bool) -> Result<()>;
    fn write_byte(&mut self, value: i8) -> Result<()>;
    fn write_i16(&mut self, value: i16) -> Result<()>;
    fn write_i32(&mut self, value: i32) -> Result<()>;
    fn write_i64(&mut self, value: i64) -> Result<()>;
    fn write_double(&mut self, value: f64) -> Result<()>;
    fn write_str(&mut self, value: &str) -> Result<()>;
    fn write_string(&mut self, value: &String) -> Result<()>;
    fn write_binary(&mut self, value: &[u8]) -> Result<()>;

    fn read_message_begin(&mut self) -> Result<(String, MessageType, i32)>;
    fn read_message_end(&mut self) -> Result<()>;

    fn read_struct_begin(&mut self) -> Result<String>;
    fn read_struct_end(&mut self) -> Result<()>;

    fn read_field_begin(&mut self) -> Result<(String, Type, i16)>;
    fn read_field_end(&mut self) -> Result<()>;

    fn read_map_begin(&mut self) -> Result<(Type, Type, i32)>;
    fn read_map_end(&mut self) -> Result<()>;

    fn read_list_begin(&mut self) -> Result<(Type, i32)>;
    fn read_list_end(&mut self) -> Result<()>;

    fn read_set_begin(&mut self) -> Result<(Type, i32)>;
    fn read_set_end(&mut self) -> Result<()>;

    fn read_bool(&mut self) -> Result<bool>;
    fn read_byte(&mut self) -> Result<i8>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_double(&mut self) -> Result<f64>;
    fn read_string(&mut self) -> Result<String>;
    fn read_binary(&mut self) -> Result<Vec<u8>>;

    fn skip(&mut self, type_: Type) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

pub trait FromNum: Sized {
    fn from_num(num: i32) -> Option<Self>;
}

pub mod helpers {
    use protocol::{Protocol, Type, MessageType, FromNum, Decode, Encode, Error};
    use Result;

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum AppResult {
        Success,
        Exception,
    }

    pub fn read_enum<F, P>(iprot: &mut P) -> Result<F>
    where F: FromNum, P: Protocol {
        let i = try!(iprot.read_i32());
        match <F as FromNum>::from_num(i) {
            Some(v) => Ok(v),
            None => Err(::Error::from(Error::ProtocolViolation)),
        }
    }

    pub fn send<W, P>(protocol: &mut P,
                         name: &str, _type: MessageType,
                         args: &W) -> Result<()>
    where W: Encode, P: Protocol {
        let cseqid: i32 = 0;    // XXX TODO FIXME
        try!(protocol.write_message_begin(name, _type, cseqid));
        try!(args.encode(protocol));
        try!(protocol.write_message_end());
        try!(protocol.flush());
        Ok(())
    }

    pub fn receive<R, P>(protocol: &mut P,
                            op: &str, result: &mut R) -> Result<AppResult>
    where R: Decode, P: Protocol {
        let (name, ty, id) = try!(protocol.read_message_begin());
        receive_body(protocol, op, result, &name, ty, id)
    }

    pub fn receive_body<R, P>(protocol: &mut P, op: &str,
                                 result: &mut R, name: &str, ty: MessageType,
                                 id: i32) -> Result<AppResult>
    where R: Decode, P: Protocol {
        match (name, ty, id) {
            (_, MessageType::Exception, _) => {
                println!("got exception");
                // TODO
                //let x = ApplicationException;
                //x.read(&mut protocol)
                //protocol.read_message_end();
                //protocol.read_end();
                //throw x
                Ok(AppResult::Exception)
            }
            // TODO: Make sure the client doesn't receive Call messages and that the server
            // doesn't receive Reply messages
            (fname, _, _) => {
                if &fname[..] == op {
                    *result = try!(R::decode(protocol));
                    try!(protocol.read_message_end());
                    Ok(AppResult::Success)
                 }
                else {
                    // FIXME: shall we err in this case?
                    try!(protocol.skip(Type::Struct));
                    try!(protocol.read_message_end());
                    Err(::Error::from(Error::ProtocolViolation))
                }
            }
        }
    }
}

