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

use std::collections::HashMap;
use transport::server::TransportServer;
use transport::Transport;
use protocol::{ProtocolFactory, Protocol};
use processor::Processor;
use server::{Server, Service};

pub struct SimpleServer<PF, TS>
    where PF: ProtocolFactory<TS::Transport>, TS: TransportServer
{
    protocol_factory: PF,
    transport_server: TS,

    processors: HashMap<&'static str, Box<Processor<PF::Protocol>>>,
}

impl<PF: ProtocolFactory<TS::Transport>, TS: TransportServer> SimpleServer<PF, TS>
    where TS::Transport: Transport {

    pub fn new(transport_server: TS, pf: PF) -> Self {
        SimpleServer {
            processors: HashMap::new(),
            protocol_factory: pf,
            transport_server: transport_server
        }
    }
}

impl<PF: ProtocolFactory<TS::Transport>, TS: TransportServer> Server for SimpleServer<PF, TS>
    where TS::Transport: Transport
{
    fn serve(&mut self) {
        loop {
            let transport = self.transport_server.accept().unwrap();
            let mut protocol = self.protocol_factory.new_protocol(transport);
            loop {
                match protocol.read_message_begin() {
                    Ok((name, _ty, id)) => {
                        match self.processors.get_mut(&name[..]) {
                            Some(mut p) => { let _ = p.process(&mut protocol, id); },
                            None => println!("missing method \"{:?}\"", name),
                        }
                    },
                    Err(err) => { println!("Failed: {:?}", err); break },
                }
            }
        }
    }
}

impl<PF, TS> Service<PF::Protocol> for SimpleServer<PF, TS>
    where PF: ProtocolFactory<TS::Transport>, TS: TransportServer
{
    fn register(&mut self, name: &'static str, processor: Box<Processor<PF::Protocol> + Send + 'static>) {
        let _ = self.processors.insert(name, processor);
    }
}