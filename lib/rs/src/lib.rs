extern crate podio;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use std::{io, fmt};
use std::error::Error as StdError;

pub use protocol::Protocol;
pub use transport::Transport;
pub use processor::Processor;

pub mod protocol;
pub mod transport;
pub mod server;
pub mod processor;

#[macro_use]
mod codegen;
mod impls;
mod compiletest;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum Error {
    /// An error occurred when reading from/writing to the underlying transport
    TransportError(io::Error),

    /// An error occurred when encoding/decoding the data
    /// (this usually indicates a bug in the library)
    ProtocolError(protocol::Error),

    /// The server code threw a user-defined exception
    UserException,
}

impl From<protocol::Error> for Error {
    fn from(err: protocol::Error) -> Error {
        Error::ProtocolError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::TransportError(err)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Thrift Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::TransportError(ref err) => Some(err),
            Error::ProtocolError(ref err) => Some(err),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Client<P,T> {
    protocol: P,
    transport: T,
    seq: i32,
}

macro_rules! proto_pass {
    ($name:ident -> $ret:ty) => {
        pub fn $name(&mut self) -> Result<$ret> {
            self.protocol.$name(&mut self.transport)
        }
    }
}

impl<P, T> Client<P, T> where P: Protocol, T: Transport {
    pub fn new(proto: P, trans: T) -> Self {
        Client { protocol: proto, transport: trans, seq: 0 }
    }

    pub fn sendcall<W: protocol::Encode>(&mut self, oneway: bool, name: &str, args: &W) -> Result<i32> {
        let ty = if oneway { protocol::MessageType::Oneway } else { protocol::MessageType::Call };
        self.seq += 1;
        try!(self.protocol.write_message_begin(&mut self.transport, name, ty, self.seq));
        try!(args.encode(&mut self.protocol, &mut self.transport));
        try!(self.protocol.write_message_end(&mut self.transport));
        try!(self.transport.flush());
        Ok(self.seq)
    }

    proto_pass!(read_message_begin -> (String, protocol::MessageType, i32));
    proto_pass!(read_message_end -> ());
    proto_pass!(read_struct_begin -> String);
    proto_pass!(read_struct_end -> ());
    proto_pass!(read_field_begin -> (String, protocol::Type, i16));
    proto_pass!(read_field_end -> ());

    pub fn skip(&mut self, ty: protocol::Type) -> Result<()> {
        self.protocol.skip(&mut self.transport, ty)
    }

    pub fn decode<D: protocol::Decode>(&mut self) -> Result<D> {
        D::decode(&mut self.protocol, &mut self.transport)
    }
}

pub trait Handler<P: Protocol, T: Transport> {
    fn handle(&mut self, proto: &mut P, trans: &mut T) -> Result<()>;
}

pub struct Service<P, T>(P, T);
impl<P: Protocol, T: Transport> Service<P, T> {
    pub fn register(&mut self, name: &str, handler: Box<Handler<P, T>>) {
        unimplemented!()
    }
}