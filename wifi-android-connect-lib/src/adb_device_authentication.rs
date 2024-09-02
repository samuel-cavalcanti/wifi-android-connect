use std::collections::HashMap;

use zeroconf::ServiceDiscovery;

use crate::client::AdbClient;

#[derive(Debug)]
pub enum AdbConnectionState {
    Unpaired(String, u32),
    Paired,
    Connected,
}

#[derive(Debug)]
pub struct AdbDeviceAuthentication {
    pub state: AdbConnectionState,
    pub known_address: HashMap<String, String>,
}

#[derive(Debug)]
pub struct AdbService {
    name: String,
    ip: String,
    port: u16,
    domain: String,
}
impl AdbService {
    pub fn address(&self) -> String {
        format!("{ip}:{port}", ip = self.ip, port = self.port)
    }
    pub fn ip(&self) -> &String {
        &self.ip
    }
}

impl From<ServiceDiscovery> for AdbService {
    fn from(value: ServiceDiscovery) -> Self {
        AdbService {
            name: value.name().into(),
            ip: value.address().into(),
            port: *value.port(),
            domain: value.domain().into(),
        }
    }
}

impl AdbDeviceAuthentication {
    pub fn new(pair_code: u32, name: String) -> AdbDeviceAuthentication {
        AdbDeviceAuthentication {
            state: AdbConnectionState::Unpaired(name, pair_code),
            known_address: HashMap::new(),
        }
    }
    pub fn is_connected(&self) -> bool {
        matches!(self.state, AdbConnectionState::Connected)
    }

    fn connect<C: AdbClient>(&mut self, address: &str, client: &C) {
        if client.adb_connect(address).is_ok() {
            self.state = AdbConnectionState::Connected
        }
    }
    fn is_not_local(domain: &str) -> bool {
        domain != "local"
    }

    pub fn on_pair<C: AdbClient>(&mut self, service: &AdbService, client: &C) {
        if let AdbConnectionState::Unpaired(name, pair_code) = &self.state {
            if !name.contains(service.name.as_str()) || Self::is_not_local(&service.domain) {
                log::trace!(
                    "service has different name or domain, service: {service:?} auth: {self:?}"
                );
                return;
            }
            if client.adb_pair(&service.address(), *pair_code).is_ok() {
                self.state = AdbConnectionState::Paired;

                let address = self.get_address(service.ip());
                if let Some(address) = address {
                    self.connect(&address, client)
                }
            }
        }
    }

    pub fn on_connect<C: AdbClient>(&mut self, service: &AdbService, client: &C) {
        if Self::is_not_local(&service.domain) {
            return;
        }
        self.known_address
            .insert(service.ip().to_string(), service.address());

        self.connect(&service.address(), client)
    }

    pub fn get_address(&self, ip: &str) -> Option<String> {
        let address = self.known_address.get(ip)?;
        Some(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::AdbClient;

    use super::{AdbDeviceAuthentication, AdbService};

    struct SuccessMock;
    struct ErrorMock;

    impl AdbClient for SuccessMock {
        fn adb_pair(&self, _address: &str, _code: u32) -> Result<(), ()> {
            Ok(())
        }

        fn adb_connect(&self, _address: &str) -> Result<(), ()> {
            Ok(())
        }
    }

    impl AdbClient for ErrorMock {
        fn adb_pair(&self, _address: &str, _code: u32) -> Result<(), ()> {
            Err(())
        }

        fn adb_connect(&self, _address: &str) -> Result<(), ()> {
            Err(())
        }
    }

    #[test]
    fn test_device_paired() {
        let mut auth = AdbDeviceAuthentication::new(10, "test".into());
        let service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "android".into(),
            port: 33001,
        };
        auth.on_connect(&service, &SuccessMock);

        assert!(auth.is_connected());
    }

    #[test]
    fn test_device_not_paired() {
        let mut auth = AdbDeviceAuthentication::new(10, "test".into());
        let service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "android".into(),
            port: 33001,
        };
        auth.on_connect(&service, &ErrorMock);

        assert!(!auth.is_connected());
    }

    #[test]
    fn test_on_connect_before_on_pair() {
        let mut auth = AdbDeviceAuthentication::new(10, "test".into());
        let connect_service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "adb-wg858lj7t959helz-si5LWZ".into(),
            port: 34003,
        };
        auth.on_connect(&connect_service, &ErrorMock);

        assert!(!auth.is_connected());

        auth.on_pair(&connect_service, &SuccessMock);

        // Should failed because the service's name is different from auth
        assert!(!auth.is_connected());

        let pair_service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "test".into(),
            port: 44123,
        };

        auth.on_pair(&pair_service, &SuccessMock);

        assert!(auth.is_connected());
    }

    #[test]
    fn test_on_pair_before_on_connect() {
        let mut auth = AdbDeviceAuthentication::new(10, "test".into());
        let pair_service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "test".into(),
            port: 33001,
        };

        let connect_service = AdbService {
            domain: "local".into(),
            ip: "123.123.0.123".into(),
            name: "adb-wg858lj7t959helz-si5LWZ".into(),
            port: 34003,
        };

        auth.on_pair(&pair_service, &SuccessMock);
        assert!(!auth.is_connected());

        auth.on_connect(&connect_service, &SuccessMock);
        assert!(auth.is_connected());
    }
}
