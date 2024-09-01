use wifi_android_connect_lib::WifiAndroidConnect;

fn main() {
    env_logger::init();
    let con = WifiAndroidConnect::default();

    println!("{}", con.qrcode_img());

    con.connect().unwrap()
}
