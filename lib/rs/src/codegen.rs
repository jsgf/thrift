#[macro_export]
macro_rules! service {
    (trait_name = $name:ident,
     processor_name = $processor_name:ident,
     client_name = $client_name:ident,
     service_methods = [
         $(
             // ThriftArgStruct -> ThriftResultStruct ThriftExnStruct= fieldname.methodname ( (arg: ty => idx)* ) -> rustreturn => [ (exn: ty => idx )* ]
             $siname:ident -> $soname:ident $sername:ident = $smfname:ident . $smname:ident( $($saname:ident: $saty:ty => $said:expr,)* ) -> $srty:ty =>
                    [ $($sename:ident $sefname:ident : $sety:ty => $seid:expr,)* ],
          )*
     ],
     parent_methods = [
         $(
             $piname:ident -> $poname:ident $pername:ident = $pmfname:ident . $pmname:ident( $($paname:ident: $paty:ty => $paid:expr,)* ) -> $prty:ty =>
                    [ $($pename:ident $pefname:ident : $pety:ty => $peid:expr,)* ],
          )*
     ],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub trait $name {
            $(fn $smname(&self, $($saname: $saty),*) -> ::std::result::Result<$srty, $soname>;)*
        }

        $(
            #[derive(Clone, Debug)]
            pub enum $sername {
                Unknown,
                $($sefname($sety),)*
            }
            impl Default for $sername {
                fn default() -> Self { $sername::Unknown }
            }
        )*

        service_processor! {
            processor_name = $processor_name,
            service_methods = [ $($siname -> $soname $sername = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename $sefname: $sety => $seid,)*],)* ],
            parent_methods = [ $($piname -> $poname $pername = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename $pefname: $pety => $peid,)*],)* ],
            bounds = [ $($boundty: $bound,)* ],
            fields = [ $($fname: $fty,)* ]
        }

        service_client! {
            client_name = $client_name,
            service_methods = [ $($siname -> $soname $sername = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename $sefname: $sety => $seid,)*],)* ],
            parent_methods = [ $($piname -> $poname $pername = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename $pefname: $pety => $peid,)*],)* ]
        }
    }
}

#[macro_export]
macro_rules! service_processor {
    (processor_name = $name:ident,
     service_methods = [
         $(
             $siname:ident -> $soname:ident $sername:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty =>
                   [ $($sename:ident $sefname:ident : $sety:ty => $seid:expr,)* ],
         )*
     ],
     parent_methods = [
         $(
             $piname:ident -> $poname:ident $pername:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty =>
                    [ $($pename:ident $pefname:ident : $pety:ty => $peid:expr,)* ],
         )*
     ],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub struct $name<$($boundty: $bound),*> {
            $($fname: $fty,)*
            _ugh: ()
        }

        $(
            strukt! { name = $siname, derive = [ Debug ],
                reqfields = {},
                optfields = { $( $saname: $saty => $said, default = Default::default(), )* }
            }
            strukt! { name = $soname, derive = [ Debug ],
                reqfields = { },
                optfields = { success: $srty => 0, default = Default::default(),
                              $( $sename: $sety => $seid, default = Default::default(), )* }
            }
         )*

        impl<$($boundty: $bound),*> $name<$($boundty),*> {
            pub fn new($($fname: $fty),*) -> Self {
                $name { $($fname: $fname,)* _ugh: () }
            }

            pub fn dispatch<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                       name: &str, ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                match name {
                    $(stringify!($smname) => self.$smname(prot, transport, ty, id),)*
                    $(stringify!($pmname) => self.$pmname(prot, transport, ty, id),)*
                    _ => Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }

            $(service_processor_method! { method = $siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename $sefname : $sety => $seid,)*] })*
            $(service_processor_method! { method = $piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename $pefname : $pety => $peid,)*] })*
        }

        impl<P: $crate::Protocol, T: $crate::Transport, $($boundty: $bound),*> $crate::Processor<P, T> for $name<$($boundty),*> {
            fn process(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()> {
                #[allow(unused_imports)]
                use $crate::Protocol;

                let (name, ty, id) = try!(protocol.read_message_begin(transport));
                self.dispatch(protocol, transport, &name, ty, id)
            }
        }
    }
}

#[macro_export]
macro_rules! service_processor_method {
    (method =
        $iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => [$($ename:ident $efname:ident : $ety:ty => $eid:expr,)*] ) => {
        fn $mname<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                               ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            try!($crate::protocol::helpers::receive_body(prot, transport, MNAME,
                                                         &mut args, MNAME, ty, id));

            // TODO: Further investigate this unwrap.
            let result = match self.$fname.$mname($(args.$aname.unwrap_or(Default::default())),*) {
                Ok(res) => $oname { success: Some(res), ..::std::default::Default::default() },
                Err(exn) => { assert!(exn.success.is_none()); exn },
            };
            try!($crate::protocol::helpers::send(prot, transport, MNAME,
                                                 $crate::protocol::MessageType::Reply, &result));

            Ok(())
        }
    }
}

#[macro_export]
macro_rules! service_client {
    (client_name = $client_name:ident,
     service_methods = [
         $(
             $siname:ident -> $soname:ident $sername:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty =>
                    [$($sename:ident $sefname:ident : $sety:ty => $seid:expr,)*],
          )*
     ],
     parent_methods = [
         $(
             $piname:ident -> $poname:ident $pername:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty =>
                    [$($pename:ident $pefname:ident : $pety:ty => $peid:expr,)*],
          )*
     ]) => {
        pub struct $client_name<P: $crate::Protocol, T: $crate::Transport> {
            pub protocol: P,
            pub transport: T
        }

        impl<P: $crate::Protocol, T: $crate::Transport> $client_name<P, T> {
            pub fn new(protocol: P, transport: T) -> Self {
                $client_name {
                    protocol: protocol,
                    transport: transport
                }
            }

            $(
                service_client_method! {
                    // ThiftArgName -> ThriftReturnName = fieldname.method( (arg: type => idx)* ) -> return => [ (exn: ty => idx) ],
                    method = $siname -> $soname $sername = $smfname.$smname ( $( $saname: $saty => $said,)* ) -> $srty => [ $($sename $sefname : $sety => $seid,)* ]
                }
            )*
            $(
                service_client_method! {
                    method = $piname -> $poname $pername = $pmfname.$pmname ( $($paname: $paty => $paid,)* ) -> $prty => [ $($pename $pefname : $pety => $peid,)* ]

                }
            )*
        }
    }
}

// Handle a method result. This is either a normal return value, or an exception.
// We're not using the normal struct unpack, because we have to pay attention to possibly
// unknown fields - if the server sends an exception we don't know about, we still have to
// consider the call a failure.
//
// XXX make a function?
#[macro_export]
macro_rules! service_client_result {
    (protocol = $protocol:expr,
     transport = $transport:expr,
     ret = $rty:ty,
     exc = $exty:ident [ $($ename:ident $efname:ident : $ety:ty => $eid:expr,)* ]) => {{
         let protocol = $protocol;
         let transport = $transport;
         let mut ret: Result<$rty, $exty> = Ok(Default::default());

         try!(protocol.read_struct_begin(transport));

         loop {
             use $crate::protocol::ThriftTyped;

             let (_, typ, id) = try!(protocol.read_field_begin(transport));

             match (typ, id) {
                 ($crate::protocol::Type::Stop, _) => break,
                 (ty, 0) if ty == <$rty as ThriftTyped>::typ() => {
                     let mut r: $rty = Default::default();
                     try!(r.decode(protocol, transport));
                     ret = Ok(r);
                 },
                 $((ty, $eid) if ty == <$ety as ThriftTyped>::typ() => {
                     let mut e: $ety = Default::default();
                     try!(e.decode(protocol, transport));
                     ret = Err($exty::$efname(e));
                 },)*
                 _ => {
                     ret = Err(Default::default());
                     try!(protocol.skip(transport, typ))
                 },
             }
             try!(protocol.read_field_end(transport));
         };

         try!(protocol.read_struct_end(transport));

         ret
     }}
}

#[macro_export]
macro_rules! service_client_method {
    // no exceptions - just return plain value
    (method = $iname:ident -> $oname:ident $ename:ident =
                $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => [ ]
    ) => {
        pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<$rty> {
            use $crate::protocol::{Decode, MessageType, Error, helpers};
            static MNAME: &'static str = stringify!($mname);

            let args = $iname { $($aname: Some($aname),)* ..::std::default::Default::default() };
            try!(helpers::send(&mut self.protocol, &mut self.transport,
                                                 MNAME, MessageType::Call, &args));

            let (name, ty, _id) = try!(self.protocol.read_message_begin(&mut self.transport));

            match ty {
                MessageType::Reply => (),
                MessageType::Exception => return Err($crate::Error::from(Error::UserException)),
                _ => return Err($crate::Error::from(Error::ProtocolViolation)),
            }
            if name != MNAME {
                return Err($crate::Error::from(Error::ProtocolViolation));
            }

            let result = service_client_result!(
                protocol = &mut self.protocol,
                transport = &mut self.transport,
                ret = $rty,
                exc = $ename [ ]
            );

            // If we got an unexpected exception, return an error, otherwise just the value
            match result {
                Err(_) => Err($crate::Error::from(Error::UserException)),
                Ok(res) => Ok(res),
            }
        }
    };

    // exceptions - return Result<T, Exn>
    (method = $iname:ident -> $oname:ident $edname:ident =
                $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty =>
                    [ $($ename:ident $efname:ident : $ety:ty => $eid:expr,)+ ]
    ) => {
        pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<::std::result::Result<$rty,$edname>> {
            use $crate::protocol::{Decode, MessageType, Error, helpers};
            static MNAME: &'static str = stringify!($mname);

            let args = $iname { $($aname: Some($aname),)* ..::std::default::Default::default() };
            try!(helpers::send(&mut self.protocol, &mut self.transport,
                                                 MNAME, MessageType::Call, &args));

            let (name, ty, _id) = try!(self.protocol.read_message_begin(&mut self.transport));

            match ty {
                MessageType::Reply => (),
                MessageType::Exception => return Err($crate::Error::from(Error::UserException)),
                _ => return Err($crate::Error::from(Error::ProtocolViolation)),
            }
            if name != MNAME {
                return Err($crate::Error::from(Error::ProtocolViolation));
            }

            let result = service_client_result!(
                protocol = &mut self.protocol,
                transport = &mut self.transport,
                ret = $rty,
                exc = $edname [ $($ename $efname : $ety => $eid,)+ ]
            );
            Ok(result)
        }
    }
}

#[macro_export]
macro_rules! strukt {
    (name = $name:ident,
     derive = [ $( $derive:ident ),* $(,)* ],
     reqfields = { $($reqfield:ident : $reqtype:ty => $reqid:expr, default = $reqdefl:expr, )* },
     optfields = { $($optfield:ident : $opttype:ty => $optid:expr, default = $optdefl:expr, )* }) => {
        #[derive(Clone$(,$derive)*)]
        pub struct $name {
            $(pub $reqfield: $reqtype,)*
            $(pub $optfield: Option<$opttype>,)*
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    $($reqfield: $reqdefl,)*
                    $($optfield: $optdefl,)*
                }
            }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::{Protocol};

                try!(protocol.write_struct_begin(transport, stringify!($name)));

                $({
                    try!(protocol.write_field_begin(transport, stringify!($reqfield), <$reqtype as ThriftTyped>::typ(), $reqid));
                    try!(self.$reqfield.encode(protocol, transport));
                    try!(protocol.write_field_end(transport));
                })*
                $({
                    if let Some(ref x) = self.$optfield {
                        try!(protocol.write_field_begin(transport, stringify!($optfield), <$opttype as ThriftTyped>::typ(), $optid));
                        try!(x.encode(protocol, transport));
                        try!(protocol.write_field_end(transport));
                    }
                })*

                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Decode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.read_struct_begin(transport));

                loop {
                    let (_, typ, id) = try!(protocol.read_field_begin(transport));

                    match (typ, id) {
                        ($crate::protocol::Type::Stop, _) => break,
                        $((ty, $reqid) if ty == <$reqtype as ThriftTyped>::typ() =>
                            try!(self.$reqfield.decode(protocol, transport)),)*
                        $((ty, $optid) if ty == <$opttype as ThriftTyped>::typ() =>
                            try!(self.$optfield.decode(protocol, transport)),)*
                        _ => try!(protocol.skip(transport, typ))
                    };

                    try!(protocol.read_field_end(transport));
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! union {
    (name = $name:ident,
     derive = [ $( $derive:ident ),* $(,)* ],
     default = $defl:expr,
     fields = { $($field:ident : $typ:ty => $id:expr, )* }) => {
        #[derive(Clone$(,$derive)*)]
        pub enum $name {
            Unknown,
            $( $field($typ), )*
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl Default for $name {
            fn default() -> Self { $defl }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::{Protocol};

                try!(protocol.write_struct_begin(transport, stringify!($name)));

                match self {
                    &$name::Unknown => (),
                    $(&$name::$field(ref val) => {
                        try!(protocol.write_field_begin(transport, stringify!($field), <$typ as ThriftTyped>::typ(), $id));
                        try!(val.encode(protocol, transport));
                        try!(protocol.write_field_end(transport));
                    },)*
                }
                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Decode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.read_struct_begin(transport));

                *self = $name::Unknown;

                loop {
                    let (_, typ, id) = try!(protocol.read_field_begin(transport));

                    match (typ, id) {
                        ($crate::protocol::Type::Stop, _) => break,
                        $((ty, $id) if ty == <$typ as ThriftTyped>::typ() => {
                            let mut v: $typ = Default::default();
                            try!(v.decode(protocol, transport));
                            *self = $name::$field(v);
                        },)*
                        _ => try!(protocol.skip(transport, typ))
                    };

                    try!(protocol.read_field_end(transport));
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! enom {
    (name = $name:ident,
     values = [$($vname:ident = $val:expr,)*],
     default = $dname:ident) => {
        #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(i32)]
        pub enum $name {
            $($vname = $val),*
        }

        impl Default for $name {
            fn default() -> Self { $name::$dname }
        }

        impl $crate::protocol::FromNum for $name {
            fn from_num(num: i32) -> Option<Self> {
                match num {
                    $($val => Some($name::$vname)),*,
                    _ => None
                }
            }
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::I32 }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                protocol.write_i32(transport, *self as i32)
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                *self = try!($crate::protocol::helpers::read_enum(protocol, transport));
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! hashmap_literal {
     ( $($key: expr => $val: expr),+ $(,)*) => (
        {
            let mut m = ::std::collections::HashMap::new();
            $(m.insert(::std::convert::From::from($key), ::std::convert::From::from($val));)+
            m
        }
    );
    ( ) => (::std::collections::HashMap::new())
}

#[macro_export]
macro_rules! btreemap_literal {
     ( $($key: expr => $val: expr),+ $(,)*) => (
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(m.insert(::std::convert::From::from($key), ::std::convert::From::from($val));)+
            m
        }
    );
    ( ) => (::std::collections::BTreeMap::new())
}

#[macro_export]
macro_rules! hashset_literal {
    ($($val:expr),+ $(,)*) => (
        {
            let mut s = ::std::collections::HashSet::new();
            $(s.insert(::std::convert::From::from($val));)+
            s
        }
    );
    ( ) => (::std::collections::HashSet::new())
}

#[macro_export]
macro_rules! btreeset_literal {
    ($($val:expr),+ $(,)*) => (
        {
            let mut s = ::std::collections::BTreeSet::new();
            $(s.insert(::std::convert::From::from($val));)+
            s
        }
    );
    ( ) => (::std::collections::BTreeSet::new())
}

#[macro_export]
macro_rules! konst {
    (const $name:ident: $ty:ty = $val:expr) => {
        lazy_static! {
            pub static ref $name: $ty = { $val };
        }
    }
}