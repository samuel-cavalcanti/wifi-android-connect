use std::str::FromStr;
use std::time::Duration;
use zeroconf::prelude::{TEventLoop, TMdnsBrowser};
use zeroconf::{EventLoop, MdnsBrowser, ServiceDiscovery, ServiceType};

use crate::adb_device_authentication::AdbService;

pub struct AdbZeroConf {
    events: [EventLoop; 2],
    #[allow(dead_code)]
    // Browsers can't dropout until the connection is finish
    browsers: [MdnsBrowser; 2],
    timeout: Duration,
}

impl AdbZeroConf {
    pub fn new(
        on_pair: Box<dyn Fn(AdbService)>,
        on_connect: Box<dyn Fn(AdbService)>,
    ) -> AdbZeroConf {
        let service_type = ServiceType::from_str("_adb-tls-pairing._tcp").unwrap();
        let mut browser_pair = MdnsBrowser::new(service_type);
        browser_pair.set_service_discovered_callback(Box::new(move |zero_s, _c| {
            if let Some(s) = zero_conf_filter_service(zero_s) {
                on_pair(s);
            }
        }));

        let mut browser_connect =
            MdnsBrowser::new(ServiceType::from_str("_adb-tls-connect._tcp").unwrap());

        browser_connect.set_service_discovered_callback(Box::new(move |zero_s, _c| {
            if let Some(s) = zero_conf_filter_service(zero_s) {
                on_connect(s);
            }
        }));

        AdbZeroConf {
            timeout: Duration::from_secs(0),
            events: [
                browser_pair.browse_services().unwrap(),
                browser_connect.browse_services().unwrap(),
            ],
            browsers: [browser_pair, browser_connect],
        }
    }

    pub fn poll(&self) -> Result<(), zeroconf::error::Error> {
        for e in self.events.iter() {
            e.poll(self.timeout)?;
        }
        Ok(())
    }
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
