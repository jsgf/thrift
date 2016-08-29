use std::io;
use std::io::Read as IoRead;
use std::io::Write as IoWrite;

use {Protocol, Transport, Result};
use protocol::{Type, MessageType};

pub use self::ProtocolAction::*;
pub use self::Primitive::*;
pub use self::Action::*;

#[derive(Debug, Clone)]
pub struct MockTransport {
    reader: io::Cursor<Vec<u8>>,
    writer: Vec<u8>,
}

impl MockTransport {
    pub fn new(buf: Vec<u8>) -> MockTransport {
        MockTransport {
            reader: io::Cursor::new(buf),
            writer: Vec::new()
        }
    }
}

impl io::Write for MockTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl io::Read for MockTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

#[derive(Debug, Clone)]
pub struct MockProtocol {
    log: Vec<ProtocolAction>,
    transport: MockTransport,
}

impl MockProtocol {
    pub fn new(transport: MockTransport) -> Self {
        MockProtocol { transport: transport, log: Vec::new() }
    }
    pub fn log(&self) -> &[ProtocolAction] { &self.log }

    fn log_action(&mut self, action: ProtocolAction) -> Result<()> {
        self.log.push(action);
        Ok(())
    }
}

macro_rules! read {
    ($selff:expr, $expected:pat, $body:expr) => {{
        if $selff.log.len() == 0 {
            panic!(concat!("Unexpected read on empty log. Expected ", stringify!($expected)))
        }

        match $selff.log.remove(0) {
             $expected => Ok($body),
             other => {
                 panic!(concat!("Unexpected read. Expected ", stringify!($expected),
                        ", encountered {:?}. Log was: {:?}"), &other, &$selff.log)
             }
        }
    }};
    ($selff:expr, $expected:pat) => { read!($selff, $expected, ()) }
}

// omg
impl Protocol for MockProtocol {
    fn write_message_begin(&mut self, name: &str,
                                         message_type: MessageType, sequence_id: i32) -> Result<()> {
        self.log_action(Message(Begin((String::from(name), message_type, sequence_id))))
    }

    fn write_message_end(&mut self) -> Result<()> {
        self.log_action(Message(End))
    }

    fn write_struct_begin(&mut self, name: &str) -> Result<()> {
         self.log_action(Struct(Begin(String::from(name))))
    }

    fn write_struct_end(&mut self) -> Result<()> {
        self.log_action(Struct(End))
    }

    fn write_field_begin(&mut self, name: &str, field_type: Type, field_id: i16) -> Result<()> {
        self.log_action(Field(Begin((String::from(name), field_type, field_id))))
    }

    fn write_field_end(&mut self) -> Result<()> {
        self.log_action(Field(End))
    }

    fn write_field_stop(&mut self) -> Result<()> {
        self.log_action(Field(Begin((String::new(), Type::Stop, 0))))
    }

    fn write_map_begin(&mut self, key_type: Type,
                                     value_type: Type, size: usize) -> Result<()> {
         self.log_action(Map(Begin((key_type, value_type, size))))
    }

    fn write_map_end(&mut self) -> Result<()> {
        self.log_action(Map(End))
    }

    fn write_list_begin(&mut self, elem_type: Type, size: usize) -> Result<()> {
        self.log_action(List(Begin((elem_type, size))))
    }

    fn write_list_end(&mut self) -> Result<()> {
        self.log_action(List(End))
    }

    fn write_set_begin(&mut self, elem_type: Type, size: usize) -> Result<()> {
        self.log_action(Set(Begin((elem_type, size))))
    }

    fn write_set_end(&mut self) -> Result<()> {
        self.log_action(Set(End))
    }

    fn write_bool(&mut self, value: bool) -> Result<()> {
        self.log_action(Prim(Bool(value)))
    }

    fn write_byte(&mut self, value: i8) -> Result<()> {
        self.log_action(Prim(Byte(value)))
    }

    fn write_i16(&mut self, value: i16) -> Result<()> {
        self.log_action(Prim(I16(value)))
    }

    fn write_i32(&mut self, value: i32) -> Result<()> {
        self.log_action(Prim(I32(value)))
    }

    fn write_i64(&mut self, value: i64) -> Result<()> {
        self.log_action(Prim(I64(value)))
    }

    fn write_double(&mut self, value: f64) -> Result<()> {
        self.log_action(Prim(Double(value)))
    }

    fn write_str(&mut self, value: &str) -> Result<()> {
        self.log_action(Prim(PString(String::from(value))))
    }

    fn write_string(&mut self, value: &String) -> Result<()> {
        self.write_str(value)
    }

    fn write_binary(&mut self, value: &[u8]) -> Result<()> {
        self.log_action(Prim(Binary(Vec::from(value))))
    }

    fn read_message_begin(&mut self) -> Result<(String, MessageType, i32)> {
        read!(self, Message(Begin((name, type_, id))), (name, type_, id))
    }

    fn read_message_end(&mut self) -> Result<()> {
        read!(self, Message(End))
    }

    fn read_struct_begin(&mut self) -> Result<String> {
        read!(self, Struct(Begin(name)), name)
    }

    fn read_struct_end(&mut self) -> Result<()> {
        read!(self, Struct(End))
    }

    fn read_field_begin(&mut self) -> Result<(String, Type, i16)> {
        read!(self, Field(Begin((name, type_, id))), (name, type_, id))
    }

    fn read_field_end(&mut self) -> Result<()> {
        read!(self, Field(End))
    }

    fn read_map_begin(&mut self) -> Result<(Type, Type, i32)> {
        read!(self, Map(Begin((keyt, valuet, len))), (keyt, valuet, len as i32))
    }

    fn read_map_end(&mut self) -> Result<()> {
        read!(self, Map(End))
    }

    fn read_list_begin(&mut self) -> Result<(Type, i32)> {
        read!(self, List(Begin((ty, len))), (ty, len as i32))
    }
    fn read_list_end(&mut self) -> Result<()> {
         read!(self, List(End))
    }

    fn read_set_begin(&mut self) -> Result<(Type, i32)> {
        read!(self, Set(Begin((ty, len))), (ty, len as i32))
    }

    fn read_set_end(&mut self) -> Result<()> {
         read!(self, Set(End))
    }

    fn read_bool(&mut self) -> Result<bool> { read!(self, Prim(Bool(val)), val) }
    fn read_byte(&mut self) -> Result<i8> { read!(self, Prim(Byte(val)), val) }
    fn read_i16(&mut self) -> Result<i16> { read!(self, Prim(I16(val)), val) }
    fn read_i32(&mut self) -> Result<i32> { read!(self, Prim(I32(val)), val) }
    fn read_i64(&mut self) -> Result<i64> { read!(self, Prim(I64(val)), val) }
    fn read_double(&mut self) -> Result<f64> { read!(self, Prim(Double(val)), val) }
    fn read_string(&mut self) -> Result<String> { read!(self, Prim(PString(string)), string) }
    fn read_binary(&mut self) -> Result<Vec<u8>> { read!(self, Prim(Binary(val)), val) }

    fn flush(&mut self) -> Result<()> { Ok(()) }

    fn skip(&mut self, _: Type) -> Result<()> {
        // TODO: Implement *checked* skipping
        if self.log.len() != 0 { self.log.pop(); }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProtocolAction {
    Message(Action<(String, MessageType, i32)>),
    Struct(Action<String>),
    Field(Action<(String, Type, i16)>),
    Map(Action<(Type, Type, usize)>),
    List(Action<(Type, usize)>),
    Set(Action<(Type, usize)>),
    Prim(Primitive)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Action<B> { Begin(B), End }

#[derive(Debug, PartialEq, Clone)]
pub enum Primitive {
    Bool(bool),
    Double(f64),
    Byte(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    PString(String),
    Binary(Vec<u8>)
}

