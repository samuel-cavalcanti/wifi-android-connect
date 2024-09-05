use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use mdns_sd::{ServiceDaemon, ServiceEvent};

use crate::{
    adb_device_authentication::AdbService, adb_mdns_discovery_service::AdbMDnsDiscoveryService,
};

pub struct AdbMdns {
    demon: ServiceDaemon,
    pair_set: Arc<Mutex<HashSet<AdbService>>>,
    connect_set: Arc<Mutex<HashSet<AdbService>>>,
}

const ADB_SERVICES: [&str; 2] = [
    "_adb-tls-pairing._tcp.local.",
    "_adb-tls-connect._tcp.local.",
];

fn event_to_adbservice(e: ServiceEvent) -> Option<AdbService> {
    if let ServiceEvent::ServiceResolved(service) = e {
        if let Some(ip) = service.get_addresses_v4().into_iter().next() {
            let adbservice = AdbService {
                name: service.get_fullname().into(),
                ip: ip.to_string(),
                port: service.get_port(),
                domain: "local".into(),
            };

            return Some(adbservice);
        }
        log::warn!("Failed to get the ip from service: {service:?}");
    }

    None
}

impl AdbMDnsDiscoveryService for AdbMdns {
    fn start(&self) -> Result<(), String> {
        let recvs = ADB_SERVICES.map(|service_type| self.demon.browse(service_type).unwrap());
        let sets = [self.pair_set.clone(), self.connect_set.clone()];

        for (recv, set) in recvs.into_iter().zip(sets) {
            std::thread::spawn(move || {
                for event in recv {
                    if let Some(adb_service) = event_to_adbservice(event) {
                        let mut set = set.lock().unwrap();
                        (*set).insert(adb_service);
                    }
                }
            });
        }

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        for service in ADB_SERVICES {
            let mut stop_result = self.demon.stop_browse(service);
            while let Err(mdns_sd::Error::Again) = stop_result {
                stop_result = self.demon.stop_browse(service);
            }

            if let Err(e) = stop_result {
                return Err(e.to_string());
            }
        }

        while let Err(mdns_sd::Error::Again) = self.demon.shutdown() {}

        Ok(())
    }

    fn adb_tls_pairing(&self) -> HashSet<AdbService> {
        (*self.pair_set.lock().unwrap()).clone()
    }

    fn adb_tls_connect(&self) -> HashSet<AdbService> {
        (*self.connect_set.lock().unwrap()).clone()
    }
}

impl AdbMdns {
    pub fn new() -> Result<AdbMdns, String> {
        let demon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        Ok(AdbMdns {
            demon,
            pair_set: Default::default(),
            connect_set: Default::default(),
        })
    }
}
