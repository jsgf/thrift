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

use protocol::{self, MessageType, Protocol, Type};
use transport::Transport;
use {Result, Error};

use podio::{ReadPodExt, WritePodExt, BigEndian};

static BINARY_PROTOCOL_VERSION_1: u16 = 0x8001;

#[derive(Copy, Clone, Debug)]
pub struct BinaryProtocol<T>(T);

impl<T: Transport> BinaryProtocol<T> {
    pub fn new(t: T) -> Self { BinaryProtocol(t) }
    
    fn write_type(&mut self, type_: Type) -> Result<()> {
        self.write_byte(type_ as i8)
    }

    fn read_type(&mut self) -> Result<Type> {
        let raw = try!(self.read_byte());
        match Type::from_num(raw as u64) {
            Some(type_) => Ok(type_),
            None => Err(Error::from(protocol::Error::ProtocolViolation("read type failed"))),
        }
    }
}

impl<T: Transport> Protocol for BinaryProtocol<T> {
    fn write_message_begin(
        &mut self,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()> {
        let version = ((BINARY_PROTOCOL_VERSION_1 as i32) << 16) | message_type as i32;
        try!(self.write_i32(version));
        try!(self.write_str(name));
        self.write_i32(sequence_id)
    }

    fn write_message_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_struct_begin(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn write_struct_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_field_begin(
        &mut self,
        _name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()> {
        try!(self.write_type(field_type));
        self.write_i16(field_id)
    }

    fn write_field_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_field_stop(&mut self) -> Result<()> {
        self.write_byte(protocol::Type::Stop as i8)
    }

    fn write_map_begin(
        &mut self,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()> {
        try!(self.write_type(key_type));
        try!(self.write_type(value_type));
        self.write_i32(size as i32)
    }

    fn write_map_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_list_begin(&mut self, elem_type: Type, size: usize) -> Result<()> {
        try!(self.write_type(elem_type));
        self.write_i32(size as i32)
    }

    fn write_list_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_set_begin(&mut self, elem_type: Type, size: usize) -> Result<()> {
        try!(self.write_type(elem_type));
        self.write_i32(size as i32)
    }

    fn write_set_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> Result<()> {
        self.write_byte(value as i8)
    }

    fn write_byte(&mut self, value: i8) -> Result<()> {
        Ok(try!(self.0.write_i8(value)))
    }

    fn write_i16(&mut self, value: i16) -> Result<()> {
        Ok(try!(self.0.write_i16::<BigEndian>(value)))
    }

    fn write_i32(&mut self, value: i32) -> Result<()> {
        Ok(try!(self.0.write_i32::<BigEndian>(value)))
    }

    fn write_i64(&mut self, value: i64) -> Result<()> {
        Ok(try!(self.0.write_i64::<BigEndian>(value)))
    }

    fn write_double(&mut self, value: f64) -> Result<()> {
        Ok(try!(self.0.write_f64::<BigEndian>(value)))
    }

    fn write_str(&mut self, value: &str) -> Result<()> {
        self.write_binary(value.as_bytes())
    }

    fn write_string(&mut self, value: &String) -> Result<()> {
        self.write_binary((&value[..]).as_bytes())
    }

    fn write_binary(&mut self, value: &[u8]) -> Result<()> {
        try!(self.write_i32(value.len() as i32));
        Ok(try!(self.0.write_all(value)))
    }

    fn read_message_begin(&mut self) -> Result<(String, MessageType, i32)> {
        let header = try!(self.read_i32());
        let version = (header >> 16) as u16;
        if version != BINARY_PROTOCOL_VERSION_1 {
            return Err(Error::from(protocol::Error::BadVersion));
        };
        let name = try!(self.read_string());
        let raw_type = header & 0xff;
        let message_type = match MessageType::from_num(raw_type as u64) {
            Some(t) => t,
            None => {
                println!("failed to read message begin raw_type {}", raw_type);
                return Err(Error::from(protocol::Error::ProtocolViolation("message begin type")))
            },
        };
        let sequence_id = try!(self.read_i32());
        Ok((name, message_type, sequence_id))
    }

    fn read_message_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_struct_begin(&mut self) -> Result<String> {
        Ok(String::new())
    }

    fn read_struct_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_field_begin(&mut self) -> Result<(String, Type, i16)> {
        let field_type = try!(self.read_type());
        let field_id = match field_type {
            protocol::Type::Stop => 0,
            _ => try!(self.read_i16()),
        };
        Ok((String::new(), field_type, field_id))
    }

    fn read_field_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_map_begin(&mut self) -> Result<(Type, Type, i32)> {
        let key_type = try!(self.read_type());
        let value_type = try!(self.read_type());
        let size = try!(self.read_i32());
        Ok((key_type, value_type, size))
    }

    fn read_map_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_list_begin(&mut self) -> Result<(Type, i32)> {
        let elem_type = try!(self.read_type());
        let size = try!(self.read_i32());
        Ok((elem_type, size))
    }

    fn read_list_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_set_begin(&mut self) -> Result<(Type, i32)> {
        let elem_type = try!(self.read_type());
        let size = try!(self.read_i32());
        Ok((elem_type, size))
    }

    fn read_set_end(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_bool(&mut self) -> Result<bool> {
        match try!(self.read_byte()) {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    fn read_byte(&mut self) -> Result<i8> {
        let transport = &mut self.0;
        Ok(try!(transport.read_i8()))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let transport = &mut self.0;
        Ok(try!(transport.read_i16::<BigEndian>()))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let transport = &mut self.0;
        Ok(try!(transport.read_i32::<BigEndian>()))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let transport = &mut self.0;
        Ok(try!(transport.read_i64::<BigEndian>()))
    }

    fn read_double(&mut self) -> Result<f64> {
        let transport = &mut self.0;
        Ok(try!(transport.read_f64::<BigEndian>()))
    }

    fn read_string(&mut self) -> Result<String> {
        let bytes = try!(self.read_binary());
        Ok(try!(String::from_utf8(bytes).map_err(|e| protocol::Error::from(e.utf8_error()))))
    }

    fn read_binary(&mut self) -> Result<Vec<u8>> {
        let len = try!(self.read_i32()) as usize;
        let mut res =  Vec::with_capacity(len);
        unsafe {
            res.set_len(len);
            try!(self.0.read_exact(&mut res[..]))
        };
        Ok(res)
    }

    fn flush(&mut self) -> Result<()> {
        try!(self.0.flush());
        Ok(())
    }

    fn skip(&mut self, type_: Type) -> Result<()> {
        match type_ {
            Type::Bool => { try!(self.read_bool()); }
            Type::I8 => { try!(self.read_byte()); }
            Type::I16 => { try!(self.read_i16()); }
            Type::I32 => { try!(self.read_i32()); }
            Type::I64 => { try!(self.read_i64()); }
            Type::Double => { try!(self.read_double()); }
            Type::String => { try!(self.read_binary()); }
            Type::Struct => {
                try!(self.read_struct_begin());
                loop {
                    let (_, field_type, _) = try!(self.read_field_begin());
                    if field_type == Type::Stop {
                        break;
                    }
                    try!(self.skip(field_type));
                    try!(self.read_field_end());
                }
                try!(self.read_struct_end());
            }
            Type::Map => {
                let (key_type, value_type, size) = try!(self.read_map_begin());
                for _ in 0..size {
                    try!(self.skip(key_type));
                    try!(self.skip(value_type));
                }
                try!(self.read_map_end());
            }
            Type::Set => {
                let (elem_type, size) = try!(self.read_set_begin());
                for _ in 0..size {
                    try!(self.skip(elem_type));
                }
                try!(self.read_set_end());
            }
            Type::List => {
                let (elem_type, size) = try!(self.read_list_begin());
                for _ in 0..size {
                    try!(self.skip(elem_type));
                }
                try!(self.read_list_end());
            }
            Type::Void => { }
            Type::Stop => { }
        };

        Ok(())
    }
}

#[cfg(test)]
pub mod test;
