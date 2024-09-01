use wifi_android_connect_lib::WifiAndroidConnect;

use clap::Parser;

/// WIFI Android Connect: A CLI tool to connect to wireless debugging using a QR code in the terminal.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct WifiAndroidConnectArgs {
    /// Name of adb service
    #[arg(short = 'n', long = "name")]
    pair_name: Option<String>,

    /// 6 digits pair code
    #[arg(short = 'c', long)]
    code: Option<u32>,

    /// show the logs
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = WifiAndroidConnectArgs::parse();
    if args.debug {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
    }

    let mut con = WifiAndroidConnect::default();

    if let Some(name) = args.pair_name {
        con.pair_name = name;
    }

    if let Some(code) = args.code {
        con.pair_code = code;
    }

    log::trace!(
        "service name: {}, pair code {}",
        con.pair_name,
        con.pair_code
    );

    match con.qrcode_img() {
        Ok(img) => println!("{img}"),
        Err(msg) => {
            println!("ERROR: {msg}");
            return;
        }
    }

    match con.connect() {
        Ok(_) => {
            println!("Connected")
        }
        Err(e) => println!("ERROR: {e}"),
    }
}
