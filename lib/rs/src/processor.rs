use protocol::Protocol;
use Result;

pub trait Processor<P: Protocol> {
    fn process(&mut self, proto: &mut P, seq: i32) -> Result<()>;
}
