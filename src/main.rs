use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{any::Any, collections::HashMap};

use adb_device::{AdbConnectionState, AdbDeviceAutentication};
use client::RustAdbClient;
use lazy_static::lazy_static;
use qrcode::{render::unicode, QrCode};
use zeroconf::prelude::{TEventLoop, TMdnsBrowser};
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

fn wifi_connect_msg(name: &str, pair_code: u32) -> String {
    format!(
        "WIFI:T:ADB;S:{hostname};P:{password:0>6};;",
        hostname = name,
        password = pair_code
    )
}

fn generate_qrcode_img(data: String) -> String {
    let code = QrCode::new(data).unwrap();
    code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}

mod adb_device;
mod client;

fn random_pair_code() -> u32 {
    rand::random::<u32>() % 1_000_000
}

lazy_static! {
    static ref ADDRESSES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref DEVICE: Mutex<AdbDeviceAutentication> = {
        Mutex::new(AdbDeviceAutentication {
            state: AdbConnectionState::Unpaired(random_pair_code()),
        })
    };
}

fn zero_conf_filter_service(
    service: zeroconf::Result<ServiceDiscovery>,
) -> Option<ServiceDiscovery> {
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
    log::info!("Service: address {}", service.address());

    if service.domain() != "local" {
        log::warn!("Ignoring service");
        return None;
    }
    Some(service)
}

fn zero_conf_on_connect(
    service: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    let service = match zero_conf_filter_service(service) {
        Some(s) => s,
        None => return,
    };

    let ip = service.address();
    let full_address = format!("{ip}:{port}", ip = ip, port = service.port());

    ADDRESSES
        .lock()
        .unwrap()
        .insert(service.address().to_string(), full_address.clone());

    let mut device = DEVICE.lock().unwrap();

    device.on_connect(&full_address, &RustAdbClient);

    log::trace!("Device state: {device:?}");
}

fn zero_conf_on_pairing(
    service: zeroconf::Result<ServiceDiscovery>,
    _context: Option<Arc<dyn Any>>,
) {
    let service = match zero_conf_filter_service(service) {
        Some(s) => s,
        None => return,
    };

    let full_address = format!("{ip}:{port}", ip = service.address(), port = service.port());

    let mut device = DEVICE.lock().unwrap();
    let addresses = ADDRESSES.lock().unwrap();
    device.on_pair(service.address(), &full_address, &RustAdbClient, &addresses);
    log::trace!("Device state: {device:?}");
}

fn main() {
    env_logger::init();
    let pair_code = DEVICE.lock().unwrap().pair_code().unwrap();

    let code = wifi_connect_msg("debug", pair_code);

    let img = generate_qrcode_img(code);

    println!("{}", img);

    let service_type = ServiceType::from_str("_adb-tls-pairing._tcp").unwrap();
    let mut browser_pair = MdnsBrowser::new(service_type);
    browser_pair.set_service_discovered_callback(Box::new(zero_conf_on_pairing));

    let mut browser_connect =
        MdnsBrowser::new(ServiceType::from_str("_adb-tls-connect._tcp").unwrap());

    browser_connect.set_service_discovered_callback(Box::new(zero_conf_on_connect));

    let event_loop = [
        browser_pair.browse_services().unwrap(),
        browser_connect.browse_services().unwrap(),
    ];

    let timeout = Duration::from_secs(0);

    loop {
        for e in &event_loop {
            let poll_result = e.poll(timeout);

            if let Err(e) = poll_result {
                log::error!("Poll error: {e:?}");
                return;
            }
        }

        if let Ok(guard) = DEVICE.try_lock() {
            if guard.is_connected() {
                break;
            }
        }
    }
}
