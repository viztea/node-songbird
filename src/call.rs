use napi::{Result, Error};
use napi_derive::napi;
use songbird::{Call, ConnectionInfo};
use songbird::id::{ChannelId, GuildId, UserId};

use crate::input::JsInput;
use crate::manager::Manager;
use crate::to_napi_error;
use crate::track_handle::JsTrackHandle;

#[napi(js_name = "Call")]
pub struct JsCall {
    inner: Call,
    pub guild_id: String,
}

#[napi(object)]
pub struct VoiceServerData {
    pub endpoint: String,
    pub token: String,
}

#[napi(object)]
pub struct VoiceStateData {
    #[napi(js_name = "session_id")]
    pub session_id: String,
    #[napi(js_name = "channel_id")]
    pub channel_id: Option<String>
}

#[napi(object)]
pub struct JsConnectionInfo {
    pub endpoint: String,
    pub token: String,
    #[napi(js_name = "user_id")]
    pub user_id: String,
    #[napi(js_name = "session_id")]
    pub session_id: String,
    #[napi(js_name = "channel_id")]
    pub channel_id: Option<String>
}

#[napi]
impl JsCall {
    #[napi(constructor)]
    pub fn new(manager: &Manager, guild_id: String) -> napi::Result<Self> {
        let call = match manager.create_call(&guild_id) {
            Some(value) => value,
            None => return Err(Error::from_reason("could not find shard for the supplied guild."))
        };

        Ok(Self { guild_id, inner: call })
    }

    #[napi]
    pub fn play(&mut self, input: &JsInput) -> JsTrackHandle {
        let track_handle = self.inner.play_input(input.create_input());
        JsTrackHandle::new(track_handle)
    }

    #[napi]
    pub fn stop(&mut self) {
        self.inner.stop()
    }

    // DRIVER-LESS

    #[napi]
    pub async fn connect(&mut self, connection_info: JsConnectionInfo) -> Result<()> {
        let channel_id = if let Some(value) = connection_info.channel_id {
            let channel_id = value.parse().map_err(to_napi_error)?;
            Some(ChannelId(channel_id))
        } else {
            None
        };

        self.inner.connect(ConnectionInfo {
            channel_id,
            endpoint: connection_info.endpoint,
            session_id: connection_info.session_id,
            token: connection_info.token,
            guild_id: GuildId(self.guild_id.parse().map_err(to_napi_error)?),
            user_id: UserId(connection_info.user_id.parse().map_err(to_napi_error)?)
        }).await.map_err(to_napi_error)
    }

    // DRIVER

    #[napi]
    pub async fn join(&mut self, channel_id: String) -> Result<()> {
        let channel = ChannelId(channel_id.parse().map_err(to_napi_error)?);
        self.inner.join(channel).await
            .map_err(to_napi_error)
            .map(|_| ())
    }

    #[napi]
    pub fn update_voice_server(&mut self, voice_server: VoiceServerData) {
        self.inner.update_server(voice_server.endpoint, voice_server.token)
    }

    #[napi]
    pub fn update_voice_state(&mut self, voice_state: VoiceStateData) -> Result<()> {
        let channel_id = if let Some(value) = voice_state.channel_id {
            let channel_id = value.parse().map_err(to_napi_error)?;
            Some(ChannelId(channel_id))
        } else {
            None
        };

        self.inner.update_state(voice_state.session_id, channel_id);
        Ok(())
    }
}
