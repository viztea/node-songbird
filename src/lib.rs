use napi::Error;
use napi_derive::napi;

mod driver;

pub mod manager;
pub mod call;
pub mod input;
pub mod track_handle;

#[napi]
pub fn init_logging() {
    env_logger::init()
}

pub(crate) fn to_napi_error<T : std::fmt::Display>(value: T) -> Error {
    Error::from_reason(format!("{}", value))
}
