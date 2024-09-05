use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;
use zeroconf::prelude::{TEventLoop, TMdnsBrowser};
use zeroconf::{EventLoop, MdnsBrowser, ServiceDiscovery, ServiceType};

use crate::adb_device_authentication::AdbService;
use crate::adb_mdns_discovery_service::AdbMDnsDiscoveryService;

pub struct AdbZeroConf {
    // Browsers can't dropout until the connection is finish
    browsers: RefCell<Option<[(MdnsBrowser, EventLoop); 2]>>,
    pair_set: Rc<RefCell<HashSet<AdbService>>>,
    connect_set: Rc<RefCell<HashSet<AdbService>>>,
}

fn zero_conf_filter_service(service: zeroconf::Result<ServiceDiscovery>) -> Option<AdbService> {
    let service = match service {
        Ok(s) => s,
        Err(e) => {
            log::error!("Error on discovery service {e}");
            return None;
        }
    };
    log::info!("On connect: {:?}", service);
    log::info!("Service: name: {}", service.name());
    log::info!("Service: port: {}", service.port());
    log::info!("service: address {}", service.address());
    log::trace!("Service domain: {}", service.domain());

    Some(AdbService::from(service))
}

impl AdbMDnsDiscoveryService for AdbZeroConf {
    fn start(&self) -> Result<(), String> {
        let mut pair_b = MdnsBrowser::new(ServiceType::from_str("_adb-tls-pairing._tcp").unwrap());
        let mut connect_b =
            MdnsBrowser::new(ServiceType::from_str("_adb-tls-connect._tcp").unwrap());

        let mut browsers = [&mut pair_b, &mut connect_b];

        let sets = [self.pair_set.clone(), self.connect_set.clone()];

        browsers.iter_mut().zip(sets).for_each(|(browser, set)| {
            browser.set_service_discovered_callback(Box::new(move |zero_s, _c| {
                if let Some(s) = zero_conf_filter_service(zero_s) {
                    let mut a = (*set).borrow_mut();
                    a.insert(s);
                }
            }));
        });

        let events = (
            browsers[0].browse_services().map_err(|e| e.to_string())?,
            browsers[1].browse_services().map_err(|e| e.to_string())?,
        );

        *self.browsers.borrow_mut() = Some([(pair_b, events.0), (connect_b, events.1)]);

        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        *self.browsers.borrow_mut() = None;
        Ok(())
    }

    fn adb_tls_pairing(&self) -> std::collections::HashSet<AdbService> {
        self.poll().unwrap();
        let set = (*self.pair_set).borrow_mut().clone();

        set
    }

    fn adb_tls_connect(&self) -> std::collections::HashSet<AdbService> {
        self.poll().unwrap();
        let set = (*self.connect_set).borrow_mut().clone();

        set
    }
}

impl AdbZeroConf {
    pub fn new() -> AdbZeroConf {
        AdbZeroConf {
            browsers: Default::default(),
            pair_set: Default::default(),
            connect_set: Default::default(),
        }
    }

    pub fn poll(&self) -> Result<(), zeroconf::error::Error> {
        if let Some(browsers) = &*self.browsers.borrow_mut() {
            for (_, e) in browsers {
                e.poll(Duration::from_secs(1))?
            }
        }
        Ok(())
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
