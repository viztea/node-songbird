use std::sync::Arc;

use napi::JsFunction;
use napi_derive::napi;
use songbird::Call;
use songbird::id::{GuildId, UserId};
use songbird::shards::{Shard, GenericSharder};

use crate::driver::Driver;
use crate::driver::shards::NodeSharder;

#[napi(object)]
#[derive(Debug)]
pub struct Fuck {
    pub shard_id: i32,
    pub payload: Fuck2
}

#[napi(object)]
#[derive(Debug)]
pub struct Fuck2 {
    #[napi(js_name = "guild_id")]
    pub guild_id: String,
    #[napi(js_name = "channel_id")]
    pub channel_id: Option<String>,
    #[napi(js_name = "self_mute")]
    pub self_mute: bool,
    #[napi(js_name = "self_deaf")]
    pub self_deaf: bool,
}

#[napi(object)]
pub struct DriverOptions {
    #[napi(ts_type = "(err: Error | null, data: Fuck) => void")]
    pub submit_voice_update: JsFunction,
    pub shard_count: i32,
}

#[napi(object)]
pub struct ManagerOptions {
    pub driver: Option<DriverOptions>,
    pub user_id: String,
}

#[napi]
pub struct Manager {
    driver: Option<Driver>,
    user_id: UserId,
}

#[napi]
impl Manager {
    #[napi(factory)]
    pub fn create(options: ManagerOptions) -> napi::Result<Self> {
        let driver = options.driver.map(|driver_options | {
            let submit_voice_update = driver_options.submit_voice_update
                .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
                .unwrap();

            Driver {
                sharder: Arc::new(NodeSharder { submit_voice_update }),
                shard_count: driver_options.shard_count as u64
            }
        });

        Ok(Self {
            user_id: UserId(options.user_id.parse().unwrap()), // TODO: handle error for this shit.
            driver
        })
    }

    pub(crate) fn create_call(&self, guild_id: &str) -> Option<Call> {
        let guild_id = to_guild_id(guild_id);

        Some(if let Some(driver) = self.driver.as_ref() {
            let shard = match driver.sharder.get_shard(
                (guild_id.0.get() >> 22) % driver.shard_count
            ) {
                Some(shard) => shard,
                None => return None,
            };

            Call::new(guild_id, Shard::Generic(shard), self.user_id)
        } else {
            Call::standalone(guild_id, self.user_id)
        })
    }
}

fn to_guild_id(str: &str) -> GuildId {
    GuildId(str.parse().unwrap())
}
