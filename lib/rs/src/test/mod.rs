use protocol::{Type, Encode, Decode};
use mock::*;

mod prim;
mod strukt;
mod enom;
mod generated;
mod consts;

pub fn encode<T: Encode>(x: &T) -> MockProtocol {
    let mut transport = MockTransport::new(vec![]);
    let mut protocol = MockProtocol::new(transport);
    x.encode(&mut protocol).unwrap();
    protocol
}

pub fn decode<T: Decode>(protocol: &mut MockProtocol) -> T {
    Decode::decode(protocol).unwrap().0
}

pub fn field_end() -> ProtocolAction {
    Field(Begin((String::new(), Type::Stop, 0)))
}

