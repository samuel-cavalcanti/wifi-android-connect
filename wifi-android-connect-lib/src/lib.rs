mod adb_device_authentication;
mod adb_zero_conf;
mod adb_zero_conf_mdns_sd;
mod client;

mod adb_mdns_discovery_service;

use adb_device_authentication::AdbDeviceAuthentication;
use adb_mdns_discovery_service::AdbMDnsDiscoveryService;
use adb_zero_conf_mdns_sd::AdbMdns;
use client::{AdbClient, RustAdbClient};
use qrcode::{render::unicode, QrCode};
use rand::Rng;

fn wifi_connect_msg(name: &str, pair_code: u32) -> Result<String, String> {
    if !(100_000..999_999).contains(&pair_code) {
        return Err("Pair code should be a 6 digits number".into());
    }
    Ok(format!(
        "WIFI:T:ADB;S:{hostname};P:{password};;",
        hostname = name,
        password = pair_code
    ))
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

pub struct WifiAndroidConnect {
    pub pair_name: String,
    pub pair_code: u32,
}

impl Default for WifiAndroidConnect {
    fn default() -> Self {
        Self {
            pair_name: "WIFI Android Connect".into(),
            pair_code: random_6_digits_pair_code(),
        }
    }
}

impl WifiAndroidConnect {
    pub fn new(pair_name: String, pair_code: u32) -> Self {
        WifiAndroidConnect {
            pair_name,
            pair_code,
        }
    }
    pub fn qrcode_img(&self) -> Result<String, String> {
        let code = wifi_connect_msg(&self.pair_name, self.pair_code)?;
        Ok(generate_qrcode_img(code))
    }
    pub fn connect(&self) -> Result<(), String> {
        let mdns = AdbMdns::new()?;
        let mut auth = AdbDeviceAuthentication::new(self.pair_code, self.pair_name.clone());

        mdns.start()?;
        let client = RustAdbClient;

        loop {
            if self.iter(&mut auth, &mdns, &client) {
                break;
            }
        }

        mdns.stop()?;

        Ok(())
    }

    fn iter(
        &self,
        auth: &mut AdbDeviceAuthentication,
        mdns: &impl AdbMDnsDiscoveryService,
        client: &impl AdbClient,
    ) -> bool {
        let pair_set = mdns.adb_tls_pairing();
        let connect_set = mdns.adb_tls_connect();

        for service in &pair_set {
            log::trace!("on pair {auth:?} {service:?}");
            auth.on_pair(service, client);
        }

        for service in &connect_set {
            log::trace!("on connect {auth:?} {service:?}");
            auth.on_connect(service, client);
        }

        auth.is_connected()
    }

    #[cfg(feature = "tokio")]
    pub async fn async_connect(&self) -> Result<(), String> {
        let mdns = AdbMdns::new()?;
        let mut auth = AdbDeviceAuthentication::new(self.pair_code, self.pair_name.clone());

        mdns.start()?;
        let client = RustAdbClient;

        loop {
            if self.iter(&mut auth, &mdns, &client) {
                break;
            }
            tokio::task::yield_now().await;
        }

        mdns.stop()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_msg_wrong_digits() {
        for code in [1, 12, 123, 1234, 12345, 1234567] {
            let msg = wifi_connect_msg("connectAndroid", code);
            assert!(msg.is_err())
        }
    }

    #[test]
    fn test_wifi_msg_6_digit() {
        let msg = wifi_connect_msg("connectAndroid", 765912).unwrap();
        assert_eq!(msg, "WIFI:T:ADB;S:connectAndroid;P:765912;;");
        let msg = wifi_connect_msg("connect Android", 123456).unwrap();
        assert_eq!(msg, "WIFI:T:ADB;S:connect Android;P:123456;;");
    }
}
