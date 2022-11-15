use napi_derive::napi;

mod shards;

pub mod manager;
pub mod player;

#[napi]
pub fn init_logging() {
    env_logger::init()
}
