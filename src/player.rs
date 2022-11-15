use napi_derive::napi;
use songbird::Call;
use songbird::id::ChannelId;
use songbird::input::YoutubeDl;

use crate::manager::Manager;

#[napi]
pub struct Player {
    pub guild_id: String,
    call: Call,
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
impl Player {
    #[napi(constructor)]
    pub fn new(manager: &Manager, guild_id: String) -> napi::Result<Self> {
        let call = manager.create_call(&guild_id).unwrap();
        Ok(Self { guild_id, call })
    }

    #[napi]
    pub async fn join(&mut self, channel_id: String) {
        let channel = ChannelId(channel_id.parse().unwrap());
        self.call.join(channel).await.expect("fuck");
    }

    #[napi]
    pub async fn play(&mut self) {
        let src = YoutubeDl::new(
            reqwest::Client::new(),
            "https://www.youtube.com/watch?v=bkPD64BBV30".to_string()
        );

        let _ = self.call.play_input(src.into());
    }

    #[napi]
    pub fn update_voice_server(&mut self, voice_server: VoiceServerData) {
        self.call.update_server(voice_server.endpoint, voice_server.token)
    }

    #[napi]
    pub fn update_voice_state(&mut self, voice_state: VoiceStateData) {
        let channel_id = voice_state.channel_id.map(|c| {
            ChannelId(c.parse().unwrap())
        });

        self.call.update_state(voice_state.session_id, channel_id)
    }
}
