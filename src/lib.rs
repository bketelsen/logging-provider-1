// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate wascc_codec as codec;

use codec::capabilities::{CapabilityProvider, Dispatcher, NullDispatcher};
use codec::core::CapabilityConfiguration;
use codec::logging::{WriteLogRequest, OP_LOG};
use wascc_codec::core::OP_CONFIGURE;
use wascc_codec::core::OP_REMOVE_ACTOR;
use wascc_codec::deserialize;

#[macro_use]
extern crate log;

use std::error::Error;
use std::sync::RwLock;

capability_provider!(LoggingProvider, LoggingProvider::new);

const CAPABILITY_ID: &str = "wascc:logging"; // TODO: change this to an appropriate capability ID

pub struct LoggingProvider {
    dispatcher: RwLock<Box<dyn Dispatcher>>,
}

impl Default for LoggingProvider {
    fn default() -> Self {
        env_logger::init();

        LoggingProvider {
            dispatcher: RwLock::new(Box::new(NullDispatcher::new())),
        }
    }
}

impl LoggingProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn configure(
        &self,
        config: impl Into<CapabilityConfiguration>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let _config = config.into();
        info!("configuring {}", CAPABILITY_ID);
        Ok(vec![])
    }
}

impl CapabilityProvider for LoggingProvider {
    fn capability_id(&self) -> &'static str {
        CAPABILITY_ID
    }

    // Invoked by the runtime host to give this provider plugin the ability to communicate
    // with actors
    fn configure_dispatch(&self, dispatcher: Box<dyn Dispatcher>) -> Result<(), Box<dyn Error>> {
        let mut lock = self.dispatcher.write().unwrap();
        *lock = dispatcher;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "wascc Logging Provider"
    }

    // Invoked by host runtime to allow an actor to make use of the capability
    // All providers MUST handle the "configure" message, even if no work will be done
    fn handle_call(&self, actor: &str, op: &str, msg: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // TIP: do not allow individual modules to attempt to send configuration,
        // only accept it from the host runtime
        if op == OP_CONFIGURE && actor == "system" {
            let cfgvals = deserialize::<CapabilityConfiguration>(msg)?;
            // setup stuff here
            match self.configure(cfgvals) {
                Ok(_) => Ok(vec![]),
                Err(e) => Err(e),
            }
        } else if op == OP_REMOVE_ACTOR && actor == "system" {
            let cfgvals = deserialize::<CapabilityConfiguration>(msg)?;
            info!("Removing actor configuration for {}", cfgvals.module);
            // tear down stuff here
            Ok(vec![])
        } else if op == OP_LOG {
            let logmsg = deserialize::<WriteLogRequest>(msg)?;
            match logmsg.level {
                1 => error!("[{}] {}", actor, logmsg.body),
                2 => warn!("[{}] {}", actor, logmsg.body),
                3 => info!("[{}] {}", actor, logmsg.body),
                4 => debug!("[{}] {}", actor, logmsg.body),
                5 => trace!("[{}] {}", actor, logmsg.body),
                _ => error!("Unknown log level: {}", logmsg.level),
            }
            Ok(vec![])
        } else {
            Err(format!("Unknown operation: {}", op).into())
        }
    }
}
