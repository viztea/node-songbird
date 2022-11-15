use std::sync::Arc;

use napi::JsFunction;
use napi_derive::napi;
use reqwest::Client;
use songbird::Call;
use songbird::id::{GuildId, UserId};
use songbird::shards::Sharder;

use crate::shards::NodeSharder;

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
pub struct ClientInfo {
    pub user_id: String,
    pub shard_count: i32
}

#[napi(object)]
pub struct ManagerOptions {
    #[napi(ts_type = "(err: Error | null, data: Fuck) => void")]
    pub submit_voice_update: JsFunction,
    pub client_info: ClientInfo,
}

#[napi]
#[derive(Clone)]
pub struct Manager {
    sharder: Arc<Sharder>,
    user_id: UserId,
    shard_count: u64,
    pub(crate) http_client: Client
}

#[napi]
impl Manager {
    #[napi(factory)]
    pub fn create(options: ManagerOptions) -> napi::Result<Self> {
        let submit_voice_update = options.submit_voice_update
            .create_threadsafe_function(0, |ctx| {
                println!("{:?}", ctx.value);
                Ok(vec![ctx.value])
            })?;

        Ok(Self {
            sharder: Arc::new(Sharder::Generic(Arc::new(NodeSharder { submit_voice_update }))),
            user_id: UserId(options.client_info.user_id.parse().unwrap()), // TODO: handle error for this shit.
            shard_count: options.client_info.shard_count as u64,
            http_client: Client::new()
        })
    }

    pub(crate) fn create_call(&self, guild_id: &str) -> Option<Call> {
        let shard = match self.sharder.get_shard(self.shard_id(guild_id)) {
            Some(shard) => shard,
            None => return None,
        };

        Some(Call::new(
            to_guild_id(guild_id),
            shard,
            self.user_id
        ))
    }

    pub(crate) fn shard_id(&self, guild_id: &str) -> u64 {
        let guild_id_u64 = guild_id.parse::<u64>().unwrap();
        (guild_id_u64 >> 22) % self.shard_count
    }
}

fn to_guild_id(str: &str) -> GuildId {
    GuildId(str.parse().unwrap())
}
