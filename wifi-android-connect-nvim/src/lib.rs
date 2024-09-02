use std::{
    cell::RefCell,
    rc::Rc,
    sync::{mpsc, Arc, OnceLock},
};

use nvim_oxi::{
    conversion::{Error as ConversionError, FromObject, ToObject},
    lua,
    serde::{Deserializer, Serializer},
    Dictionary, Function, Object,
};
use serde::{Deserialize, Serialize};
use wifi_android_connect_lib::WifiAndroidConnect;

#[nvim_oxi::plugin]
fn libwifi_android_connect_nvim() -> Dictionary {
    let conn = wifi_android_connect_lib::WifiAndroidConnect::default();
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

fn setup(conn: Rc<RefCell<WifiAndroidConnect>>) -> impl Fn(Setup) {
    move |setup| {
        let mut conn = (*conn).borrow_mut();
        if let Some(pair_code) = setup.pair_code {
            conn.pair_code = pair_code
        }

        if let Some(pair_name) = setup.pair_name {
            conn.pair_name = pair_name
        }
    }
}
// using for tests..
fn qrcode(conn: Rc<RefCell<WifiAndroidConnect>>) -> impl Fn(()) -> String {
    move |()| {
        let conn = (*conn).borrow_mut();

        match conn.qrcode_img() {
            Ok(qrcode_img) => qrcode_img,
            Err(error_msg) => error_msg,
        }
    }
}
fn connect(conn: WifiAndroidConnect) -> impl Fn(Function<String, ()>) -> String {
    let conn = Arc::new(conn);

    log::trace!("pair name {} code {}", conn.pair_name, conn.pair_code);
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    let runtime = RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap());

    move |calback| {
        let conn = conn.clone();
        let qrcode = conn.qrcode_img().unwrap();

        let (tx, rx) = mpsc::channel::<String>();

        let handle = nvim_oxi::libuv::AsyncHandle::new(move || {
            let msg = rx.recv().unwrap();
            calback.call(msg).unwrap();
        })
        .unwrap();

        runtime.spawn(async move {
            let msg = match conn.connect() {
                Ok(_) => "Connected".into(),
                Err(e) => e,
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
