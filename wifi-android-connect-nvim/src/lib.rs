use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, OnceLock},
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

const DEFAULT_TIMEOUT: u64 = 2 * 60;

#[nvim_oxi::plugin]
fn libwifi_android_connect_nvim() -> Dictionary {
    let s: Rc<RefCell<Setup>> = Default::default();

    let qrcode_fun = Object::from(Function::from_fn(qrcode(s.clone())));
    let setup_fun = Object::from(Function::from_fn(setup(s.clone())));

    let connect_fun = Object::from(Function::from_fn(connect(s)));

    Dictionary::from_iter([
        ("setup", setup_fun),
        ("qrcode", qrcode_fun),
        ("connect", connect_fun),
    ])
}

fn setup(s: Rc<RefCell<Setup>>) -> impl Fn(Setup) {
    move |setup_user| {
        *(*s).borrow_mut() = setup_user;
    }
}
// using for tests..
fn qrcode(s: Rc<RefCell<Setup>>) -> impl Fn(()) -> String {
    move |()| {
        let setup = &*(*s).borrow_mut();
        let conn = WifiAndroidConnect::from(setup);

        match conn.qrcode_img() {
            Ok(qrcode_img) => qrcode_img,
            Err(error_msg) => error_msg,
        }
    }
}
fn connect(setup: Rc<RefCell<Setup>>) -> impl Fn(Function<String, ()>) -> String {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    let runtime = RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap());

    move |calback| {
        let setup = &*(*setup).borrow_mut();
        let conn = WifiAndroidConnect::from(setup);
        let timeout = setup.timeout_in_seconds.unwrap_or(DEFAULT_TIMEOUT);
        let qrcode = conn.qrcode_img().unwrap();

        let (tx, rx) = mpsc::channel::<String>();

        let handle = nvim_oxi::libuv::AsyncHandle::new(move || {
            let msg = rx.recv().unwrap();
            calback.call(msg).unwrap();
        })
        .unwrap();

        runtime.spawn(async move {
            let timeout = tokio::time::timeout(Duration::from_secs(timeout), async {
                conn.async_connect().await
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

impl From<Setup> for WifiAndroidConnect {
    fn from(value: Setup) -> Self {
        let mut conn = WifiAndroidConnect::default();
        if let Some(code) = value.pair_code {
            conn.pair_code = code;
        }
        if let Some(name) = value.pair_name {
            conn.pair_name = name;
        }

        conn
    }
}

impl From<&Setup> for WifiAndroidConnect {
    fn from(value: &Setup) -> Self {
        let mut conn = WifiAndroidConnect::default();
        if let Some(code) = value.pair_code {
            conn.pair_code = code;
        }
        if let Some(name) = &value.pair_name {
            conn.pair_name = name.clone();
        }

        conn
    }
}

#[derive(Serialize, Deserialize)]
struct Setup {
    pair_name: Option<String>,
    pair_code: Option<u32>,
    timeout_in_seconds: Option<u64>,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            pair_code: None,
            pair_name: None,
            timeout_in_seconds: Some(DEFAULT_TIMEOUT),
        }
    }
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
