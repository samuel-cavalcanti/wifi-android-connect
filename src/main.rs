mod adb_device_authentication;
mod adb_zero_conf;
mod client;

use std::cell::RefCell;
use std::rc::Rc;

use adb_device_authentication::{AdbDeviceAuthentication, AdbService};
use adb_zero_conf::AdbZeroConf;
use client::RustAdbClient;
use qrcode::{render::unicode, QrCode};

fn wifi_connect_msg(name: &str, pair_code: u32) -> String {
    format!(
        "WIFI:T:ADB;S:{hostname};P:{password:0>6};;",
        hostname = name,
        password = pair_code
    )
}

#[test]
fn test_wifi_msg() {
    let msg = wifi_connect_msg("connectAndroid", 5);

    assert_eq!(msg, "WIFI:T:ADB;S:connectAndroid;P:000005;;");

    let msg = wifi_connect_msg("connectAndroid", 765912);
    assert_eq!(msg, "WIFI:T:ADB;S:connectAndroid;P:765912;;");

    let msg = wifi_connect_msg("debug", 912);
    assert_eq!(msg, "WIFI:T:ADB;S:debug;P:000912;;");
}

fn generate_qrcode_img(data: String) -> String {
    let code = QrCode::new(data).unwrap();
    code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}

fn random_pair_code() -> u32 {
    rand::random::<u32>() % 1_000_000
}

fn on_pair(auth: Rc<RefCell<AdbDeviceAuthentication>>) -> impl Fn(AdbService) {
    move |s| {
        auth.borrow_mut().on_pair(&s, &RustAdbClient);
        log::trace!("Auth state: {auth:?}");
        log::trace!("service: {s:?}");
    }
}
fn on_connect(auth: Rc<RefCell<AdbDeviceAuthentication>>) -> impl Fn(AdbService) {
    move |s| {
        auth.borrow_mut().on_connect(&s, &RustAdbClient);
        log::trace!("Auth state: {auth:?}");
        log::trace!("service: {s:?}");
    }
}

fn main() {
    env_logger::init();

    let (pair_name, pair_code) = ("connectAndroid", random_pair_code());
    let auth = AdbDeviceAuthentication::new(pair_code, pair_name.to_string());
    let auth = RefCell::new(auth);
    let auth = Rc::new(auth);

    let code = wifi_connect_msg(pair_name, pair_code);

    let img = generate_qrcode_img(code);

    println!("{}", img);

    let zeroconf = AdbZeroConf::new(
        Box::new(on_pair(auth.clone())),
        Box::new(on_connect(auth.clone())),
    );

    loop {
        if let Err(e) = zeroconf.poll() {
            log::error!("Poll error: {e:?}");
            return;
        }

        if auth.borrow_mut().is_connected() {
            break;
        }
    }
}
