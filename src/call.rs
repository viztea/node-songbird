use napi::JsFunction;
use napi_derive::napi;
use songbird::Call;
use songbird::id::ChannelId;
use crate::input::JsInput;

use crate::manager::Manager;
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

#[napi]
impl JsCall {
    #[napi(constructor)]
    pub fn new(manager: &Manager, guild_id: String) -> napi::Result<Self> {
        let call = manager.create_call(&guild_id).unwrap();
        Ok(Self { guild_id, inner: call })
    }

    #[napi]
    pub async fn join(&mut self, channel_id: String) {
        let channel = ChannelId(channel_id.parse().unwrap());
        self.inner.join(channel).await.expect("fuck");
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

    #[napi]
    pub fn update_voice_server(&mut self, voice_server: VoiceServerData) {
        self.inner.update_server(voice_server.endpoint, voice_server.token)
    }

    #[napi]
    pub fn update_voice_state(&mut self, voice_state: VoiceStateData) {
        let channel_id = voice_state.channel_id.map(|c| {
            ChannelId(c.parse().unwrap())
        });

        self.inner.update_state(voice_state.session_id, channel_id)
    }
}
