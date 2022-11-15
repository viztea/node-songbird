extern crate core;

use napi_derive::napi;

mod shards;

pub mod manager;
pub mod call;
pub mod input;
pub mod track_handle;

#[napi]
pub fn init_logging() {
    env_logger::init()
}
