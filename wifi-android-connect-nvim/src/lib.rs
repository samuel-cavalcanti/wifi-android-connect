use nvim_oxi::{
    conversion::{Error as ConversionError, FromObject, ToObject},
    lua, print,
    serde::{Deserializer, Serializer},
    Dictionary, Function, Object,
};
use serde::{Deserialize, Serialize};

#[nvim_oxi::plugin]
fn android_connect() -> Dictionary {
    Dictionary::from_iter([("setup", Function::from_fn(setup))])
}

fn setup(setup: Setup) {
    let mut conn = wifi_android_connect_lib::WifiAndroidConnect::default();

    if let Some(pair_code) = setup.pair_code {
        conn.pair_code = pair_code
    }

    if let Some(pair_name) = setup.pair_name {
        conn.pair_name = pair_name
    }

    print!("conn: name {} code {}", conn.pair_name, conn.pair_code);
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
