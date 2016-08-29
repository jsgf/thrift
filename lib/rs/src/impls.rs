pub use protocol::{self, Encode, Decode, Type, ThriftTyped};
pub use {Protocol, Result, Error};

pub use std::collections::{HashSet, HashMap, BTreeSet, BTreeMap};
pub use std::hash::Hash;

impl ThriftTyped for bool { fn typ() -> Type { Type::Bool } }
impl ThriftTyped for i8  { fn typ() -> Type { Type::I8 } }
impl ThriftTyped for i16 { fn typ() -> Type { Type::I16 } }
impl ThriftTyped for i32 { fn typ() -> Type { Type::I32 } }
impl ThriftTyped for i64 { fn typ() -> Type { Type::I64 } }
impl ThriftTyped for f64 { fn typ() -> Type { Type::Double } }
impl ThriftTyped for () { fn typ() -> Type { Type::Void } }
impl ThriftTyped for String { fn typ() -> Type { Type::String } }
impl ThriftTyped for Vec<u8> { fn typ() -> Type { Type::String } }
impl<T: ThriftTyped> ThriftTyped for Vec<T> { fn typ() -> Type { Type::List } }
impl<T: ThriftTyped> ThriftTyped for Option<T> { fn typ() -> Type { T::typ() } }
impl<T: ThriftTyped> ThriftTyped for HashSet<T> { fn typ() -> Type { Type::Set } }
impl<T: ThriftTyped> ThriftTyped for BTreeSet<T> { fn typ() -> Type { Type::Set } }
impl<K: ThriftTyped, V: ThriftTyped> ThriftTyped for HashMap<K, V> { fn typ() -> Type { Type::Map } }
impl<K: ThriftTyped, V: ThriftTyped> ThriftTyped for BTreeMap<K, V> { fn typ() -> Type { Type::Map } }

impl Encode for Vec<u8> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_binary(&self[..]));

        Ok(())
    }
}

impl<X: Encode> Encode for Vec<X> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_list_begin(X::typ(), self.len()));

        for el in self {
            try!(el.encode(protocol));
        }

        try!(protocol.write_list_end());

        Ok(())
    }
}

impl<X: Encode + Hash + Eq> Encode for HashSet<X> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_set_begin(X::typ(), self.len()));

        for el in self {
            try!(el.encode(protocol));
        }

        try!(protocol.write_set_end());

        Ok(())
    }
}

impl<X: Encode + Ord + Eq> Encode for BTreeSet<X> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_set_begin(X::typ(), self.len()));

        for el in self {
            try!(el.encode(protocol));
        }

        try!(protocol.write_set_end());

        Ok(())
    }
}

impl<K: Encode + Hash + Eq, V: Encode> Encode for HashMap<K, V> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_map_begin(K::typ(), V::typ(), self.len()));

        for (k, v) in self.iter() {
            try!(k.encode(protocol));
            try!(v.encode(protocol));
        }

        try!(protocol.write_map_end());

        Ok(())
    }
}

impl<K: Encode + Ord + Eq, V: Encode> Encode for BTreeMap<K, V> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_map_begin(K::typ(), V::typ(), self.len()));

        for (k, v) in self.iter() {
            try!(k.encode(protocol));
            try!(v.encode(protocol));
        }

        try!(protocol.write_map_end());

        Ok(())
    }
}

impl<X: Encode> Encode for Option<X> {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        self.as_ref().map(|this| this.encode(protocol)).unwrap_or(Ok(()))
    }
}

impl Encode for String {
    fn encode<P>(&self, protocol: &mut P) -> Result<()>
    where P: Protocol {
        try!(protocol.write_string(self));
        Ok(())
    }
}

impl Encode for () {
    fn encode<P>(&self, _: &mut P) -> Result<()>
    where P: Protocol { Ok(()) }
}

macro_rules! prim_encode {
    ($($T:ty => $method:ident),*) => {
        $(impl Encode for $T {
            fn encode<P>(&self, protocol: &mut P) -> Result<()>
            where P: Protocol {
                try!(protocol.$method(*self));
                Ok(())
            }
        })*
    }
}

prim_encode! {
    bool => write_bool, i8 => write_byte, i16 => write_i16,
    i32 => write_i32, i64 => write_i64, f64 => write_double
}

impl Decode for Vec<u8> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        protocol.read_binary().map(|v| (v, false))
    }
}

impl<X: Decode> Decode for Vec<X> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (typ, len) = try!(protocol.read_list_begin());

        if typ == X::typ() {
            let mut ret = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let (v, _) = try!(X::decode(protocol));
                ret.push(v)
            }
            try!(protocol.read_list_end());
            Ok((ret, false))
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation("vec decode type")))
        }
    }
}

impl<X: Decode + Eq + Hash> Decode for HashSet<X> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (typ, len) = try!(protocol.read_set_begin());

        if typ == X::typ() {
            let mut ret = HashSet::with_capacity(len as usize);
            for _ in 0..len {
                let (v, _) = try!(X::decode(protocol));
                ret.insert(v);
            }
            try!(protocol.read_set_end());
            Ok((ret, false))
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation("hashset decode type")))
        }
    }
}

impl<X: Decode + Ord + Hash> Decode for BTreeSet<X> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (typ, len) = try!(protocol.read_set_begin());

        if typ == X::typ() {
            let mut ret = BTreeSet::new();
            for _ in 0..len {
                let (v, _) = try!(X::decode(protocol));
                ret.insert(v);
            }
            try!(protocol.read_set_end());
            Ok((ret, false))
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation("btreeset decode type")))
        }
    }
}

impl<K: Decode + Eq + Hash, V: Decode> Decode for HashMap<K, V> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (ktyp, vtyp, len) = try!(protocol.read_map_begin());

        if ktyp == K::typ() && vtyp == V::typ() {
            let mut ret = HashMap::with_capacity(len as usize);
            for _ in 0..len {
                let (key, _) = try!(K::decode(protocol));
                let (value, _) = try!(V::decode(protocol));
                ret.insert(key, value);
            }

            try!(protocol.read_map_end());
            Ok((ret, false))
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation("hashmap decode type")))
        }
    }
}

impl<K: Decode + Ord + Hash, V: Decode> Decode for BTreeMap<K, V> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (ktyp, vtyp, len) = try!(protocol.read_map_begin());

        if ktyp == K::typ() && vtyp == V::typ() {
            let mut ret = BTreeMap::new();
            for _ in 0..len {
                let (key, _) = try!(K::decode(protocol));
                let (value, _) = try!(V::decode(protocol));
                ret.insert(key, value);
            }

            try!(protocol.read_map_end());
            Ok((ret, false))
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation("btreemap decode type")))
        }
    }
}

impl<X: Decode> Decode for Option<X> {
    fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
    where P: Protocol {
        let (v, _) = try!(X::decode(protocol));
        Ok((Some(v), false))
    }
}

impl Decode for () {
    fn decode<P>(_: &mut P) -> Result<(Self, bool)>
    where P: Protocol { Ok(((), false)) }
}

macro_rules! prim_decode {
    ($($T:ty => $method:ident),*) => {
        $(impl Decode for $T {
            fn decode<P>(protocol: &mut P) -> Result<(Self, bool)>
            where P: Protocol {
                Ok((try!(protocol.$method()), false))
            }
        })*
    }
}

prim_decode! {
    bool => read_bool, i8 => read_byte, i16 => read_i16,
    i32 => read_i32, i64 => read_i64, f64 => read_double,
    String => read_string
}

