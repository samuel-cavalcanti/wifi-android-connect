use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, Arc, OnceLock},
    time::Duration,
};

use nvim_oxi::{
    conversion::{Error as ConversionError, FromObject, ToObject},
    lua,
    serde::{Deserializer, Serializer},
    Dictionary, Function, Object,
};
use serde::{Deserialize, Serialize};
use wifi_android_connect_lib::WifiAndroidConnect;

struct WifiAndroidConnectPlugin {
    conn: WifiAndroidConnect,
    timeout: u64,
}

impl Default for WifiAndroidConnectPlugin {
    fn default() -> Self {
        Self {
            conn: Default::default(),
            timeout: 5,
        }
    }
}

#[nvim_oxi::plugin]
fn libwifi_android_connect_nvim() -> Dictionary {
    let conn = WifiAndroidConnectPlugin::default();
    let conn = RefCell::new(conn);
    let conn = Rc::new(conn);

    let qrcode_fun = Object::from(Function::from_fn(qrcode(conn.clone())));
    let setup_fun = Object::from(Function::from_fn(setup(conn.clone())));

    let conn = (*conn).take();
    let connect_fun = Object::from(Function::from_fn(connect(conn)));

    Dictionary::from_iter([
        ("setup", setup_fun),
        ("qrcode", qrcode_fun),
        ("connect", connect_fun),
    ])
}

fn setup(conn: Rc<RefCell<WifiAndroidConnectPlugin>>) -> impl Fn(Setup) {
    move |setup| {
        let mut plugin = (*conn).borrow_mut();

        if let Some(pair_code) = setup.pair_code {
            plugin.conn.pair_code = pair_code
        }

        if let Some(pair_name) = setup.pair_name {
            plugin.conn.pair_name = pair_name
        }
        if let Some(timeout) = setup.timeout_in_seconds {
            plugin.timeout = timeout;
        }
    }
}
// using for tests..
fn qrcode(conn: Rc<RefCell<WifiAndroidConnectPlugin>>) -> impl Fn(()) -> String {
    move |()| {
        let conn = &mut (*conn).borrow_mut().conn;

        match conn.qrcode_img() {
            Ok(qrcode_img) => qrcode_img,
            Err(error_msg) => error_msg,
        }
    }
}
fn connect(plugin: WifiAndroidConnectPlugin) -> impl Fn(Function<String, ()>) -> String {
    let plugin = Arc::new(plugin);

    log::trace!(
        "pair name {} code {}",
        plugin.conn.pair_name,
        plugin.conn.pair_code
    );
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    let runtime = RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap());

    move |calback| {
        let plugin = plugin.clone();
        let qrcode = plugin.conn.qrcode_img().unwrap();

        let (tx, rx) = mpsc::channel::<String>();

        let handle = nvim_oxi::libuv::AsyncHandle::new(move || {
            let msg = rx.recv().unwrap();
            calback.call(msg).unwrap();
        })
        .unwrap();

        runtime.spawn(async move {
            let timeout = tokio::time::timeout(Duration::from_secs(plugin.timeout), async {
                plugin.conn.async_connect().await
            })
            .await;

            let msg = match timeout {
                Ok(connect_result) => match connect_result {
                    Ok(_) => "Connected".into(),
                    Err(e) => e,
                },
                Err(_) => "Timeout".into(),
            };

            tx.send(msg).unwrap();
            handle.send().unwrap();
        });

        qrcode
    }
}

#[derive(Serialize, Deserialize)]
struct Setup {
    pair_name: Option<String>,
    pair_code: Option<u32>,
    timeout_in_seconds: Option<u64>,
}

impl FromObject for Setup {
    fn from_object(obj: Object) -> Result<Self, ConversionError> {
        Self::deserialize(Deserializer::new(obj)).map_err(Into::into)
    }
}

impl ToObject for Setup {
    fn to_object(self) -> Result<Object, ConversionError> {
        self.serialize(Serializer::new()).map_err(Into::into)
    }
}

impl lua::Poppable for Setup {
    unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
        let obj = Object::pop(lstate)?;
        Self::from_object(obj).map_err(lua::Error::pop_error_from_err::<Self, _>)
    }
}

impl lua::Pushable for Setup {
    unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
        self.to_object()
            .map_err(lua::Error::push_error_from_err::<Self, _>)?
            .push(lstate)
    }
}
