use std::net::SocketAddrV4;

pub trait AdbClient {
    fn adb_pair(&self, address: &str, code: u32) -> Result<(), ()>;
    fn adb_connect(&self, address: &str) -> Result<(), ()>;
}

pub struct RustAdbClient;
impl AdbClient for RustAdbClient {
    fn adb_pair(&self, address: &str, code: u32) -> Result<(), ()> {
        let mut server = adb_client::ADBServer::default();

        let ipv4 = match address.parse::<SocketAddrV4>() {
            Ok(add) => add,
            Err(e) => {
                log::error!("Unable to parse address: {address}");
                log::error!("Error: {e:?}");
                return Err(());
            }
        };

        match server.pair(ipv4, code) {
            Ok(_ok) => {
                log::info!("Device paired");
                Ok(())
            }
            Err(e) => {
                log::error!("Pair Error: {e:?}");
                if let adb_client::RustADBError::ADBRequestFailed(s) = e {
                    log::error!("msg: {s}");
                }
                Err(())
            }
        }
    }

    fn adb_connect(&self, address: &str) -> Result<(), ()> {
        let ipv4 = match address.parse::<SocketAddrV4>() {
            Ok(add) => add,
            Err(e) => {
                log::error!("Unable to parse address: {address}");
                log::error!("Error: {e:?}");
                return Err(());
            }
        };

        let mut server = adb_client::ADBServer::default();

        match server.connect_device(ipv4) {
            Ok(_ok) => {
                log::info!("Connected Device address: {address}");
                Ok(())
            }
            Err(e) => {
                log::error!("Error: {e:?}");
                if let adb_client::RustADBError::ADBRequestFailed(msg) = e {
                    if msg.contains("already connected") {
                        return Ok(());
                    }
                }
                log::error!("Unable to Connect Device address: {address}");
                Err(())
            }
        }
    }
}
