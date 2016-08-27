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

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use threadpool::ThreadPool;

use transport::server::TransportServer;
use protocol::{Protocol, ProtocolFactory};
use processor::Processor;
use server::{Server, Service};

pub struct ThreadedServer<PF, TS>
    where PF: ProtocolFactory<TS::Transport>,
          TS: TransportServer,
          PF::Protocol: Send + 'static
{
    inner: Arc<Mutex<ThreadedServerInner<PF, TS>>>,
    pool: ThreadPool,
    protocol_factory: PF,
    transport_server: TS
}

struct ThreadedServerInner<PF, TS>
    where PF: ProtocolFactory<TS::Transport>,
          TS: TransportServer,
          PF::Protocol: Send + 'static
{
    processors: HashMap<&'static str, Box<Processor<PF::Protocol> + 'static + Send>>,
}

impl<PF, TS> ThreadedServer<PF, TS>
where TS: TransportServer + 'static,
      PF: ProtocolFactory<TS::Transport> + 'static,
      PF::Protocol: Send + 'static
{
    pub fn new(server: TS, factory: PF, threads: usize) -> Self {
        ThreadedServer {
            protocol_factory: factory,
            transport_server: server,
            pool: ThreadPool::new(threads),
            inner: Arc::new(Mutex::new(ThreadedServerInner {
                processors: HashMap::new(),
            })),
        }
    }
}

impl<PF, TS> Server for ThreadedServer<PF, TS>
where TS: TransportServer + 'static,
      PF: ProtocolFactory<TS::Transport> + 'static,
      PF::Protocol: Send + 'static
{
    fn serve(&mut self) {
        loop {
            let transport = self.transport_server.accept().expect("Accept failed");
            let mut protocol = self.protocol_factory.new_protocol(transport);
            let processors = self.inner.clone();

            self.pool.execute(move || {
                loop {
                    match protocol.read_message_begin() {
                        Ok((name, _ty, id)) => {
                            let mut inner = processors.lock().expect("lock");
                            match inner.processors.get_mut(&name[..]) {
                                Some(mut p) => { let _ = p.process(&mut protocol, id); },
                                None => { println!("unknown method {}", name); /* XXX read rest */ }
                            }
                        },
                        Err(err) => { println!("read failed: {:?}", err); break },
                    }
                }
            })
        }
    }
}

impl<PF, TS> Service<PF::Protocol> for ThreadedServer<PF, TS>
    where PF: ProtocolFactory<TS::Transport>,
          TS: TransportServer,
          PF::Protocol: Send
{
    fn register(&mut self, name: &'static str, processor: Box<Processor<PF::Protocol> + Send + 'static>) {
        let mut processors = self.inner.lock().expect("lock");
        let _ = processors.processors.insert(name, processor);
    }
}