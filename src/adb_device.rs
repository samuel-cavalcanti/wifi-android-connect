use std::collections::HashMap;

use crate::client::AdbClient;

#[derive(Debug)]
pub enum AdbConnectionState {
    Unpaired(u32),
    Paired,
    Connected,
}

#[derive(Debug)]
pub struct AdbDeviceAutentication {
    pub state: AdbConnectionState,
}

impl AdbDeviceAutentication {
    pub fn pair_code(&self) -> Option<u32> {
        match self.state {
            AdbConnectionState::Unpaired(code) => Some(code),
            _ => None,
        }
    }
    pub fn is_paried(&self) -> bool {
        match self.state {
            AdbConnectionState::Paired => true,
            _ => false,
        }
    }
    pub fn is_connected(&self) -> bool {
        match self.state {
            AdbConnectionState::Connected => true,
            _ => false,
        }
    }

    pub fn on_connect<C: AdbClient>(&mut self, address: &str, client: &C) {
        if client.adb_connect(address).is_ok() {
            self.state = AdbConnectionState::Connected
        }
    }

    pub fn on_pair<C: AdbClient>(
        &mut self,
        ip: &str,
        address: &str,
        client: &C,
        devices: &HashMap<String, String>,
    ) {
        if let AdbConnectionState::Unpaired(pair_code) = self.state {
            if client.adb_pair(address, pair_code).is_ok() {
                self.state = AdbConnectionState::Paired;
                if let Some(add) = devices.get(ip) {
                    self.on_connect(add, client)
                }
            }
        }
    }
}
