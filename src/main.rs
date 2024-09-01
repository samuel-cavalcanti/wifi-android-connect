mod adb_device_authentication;
mod adb_zero_conf;
mod client;

use std::cell::RefCell;
use std::rc::Rc;

use adb_device_authentication::{AdbDeviceAuthentication, AdbService};
use adb_zero_conf::AdbZeroConf;
use client::RustAdbClient;
use qrcode::{render::unicode, QrCode};
use rand::Rng;

fn wifi_connect_msg(name: &str, pair_code: u32) -> String {
    assert!(
        (100_000..999_999).contains(&pair_code),
        "Should be a 6 digits number"
    );
    format!(
        "WIFI:T:ADB;S:{hostname};P:{password};;",
        hostname = name,
        password = pair_code
    )
}

#[test]
#[should_panic]
fn test_wifi_msg_1_digit() {
    let _msg = wifi_connect_msg("connectAndroid", 5);
}

#[test]
#[should_panic]
fn test_wifi_msg_5_digits() {
    let _msg = wifi_connect_msg("connectAndroid", 12345);
}

#[test]
fn test_wifi_msg_6_digit() {
    let msg = wifi_connect_msg("connectAndroid", 765912);
    assert_eq!(msg, "WIFI:T:ADB;S:connectAndroid;P:765912;;");
    let msg = wifi_connect_msg("connect Android", 123456);
    assert_eq!(msg, "WIFI:T:ADB;S:connect Android;P:123456;;");
}

#[test]
#[should_panic]
fn test_wifi_msg_7_digits() {
    let _msg = wifi_connect_msg("connectAndroid", 1234567);
}

fn generate_qrcode_img(data: String) -> String {
    let code = QrCode::new(data).unwrap();
    code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}

fn random_6_digits_pair_code() -> u32 {
    rand::thread_rng().gen_range(100_000..999_999)
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

    let (pair_name, pair_code) = ("WIFI Android Connect", random_6_digits_pair_code());
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
