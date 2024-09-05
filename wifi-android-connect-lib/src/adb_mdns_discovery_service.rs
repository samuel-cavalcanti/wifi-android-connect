use std::collections::HashSet;

use crate::adb_device_authentication::AdbService;

pub trait AdbMDnsDiscoveryService {
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn adb_tls_pairing(&self) -> HashSet<AdbService>;
    fn adb_tls_connect(&self) -> HashSet<AdbService>;
}
