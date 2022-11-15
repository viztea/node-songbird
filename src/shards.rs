use std::sync::Arc;

use async_trait::async_trait;
use napi::{threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode}};

use songbird::shards::{GenericSharder, VoiceUpdate};

use crate::manager::{Fuck, Fuck2};

type SubmitVoiceUpdate = ThreadsafeFunction<Fuck, ErrorStrategy::CalleeHandled>;

pub struct NodeSharder {
    pub submit_voice_update: SubmitVoiceUpdate
}

#[async_trait]
impl GenericSharder for NodeSharder {
    fn get_shard(&self, shard_id:u64) -> Option<Arc<dyn VoiceUpdate+Send+Sync>> {
        let shard = NodeShard {
            submit_voice_update: self.submit_voice_update.clone(),
            shard_id
        };

        Some(Arc::new(shard))
    }
}

pub struct NodeShard {
    submit_voice_update: SubmitVoiceUpdate,
    shard_id: u64
}

#[async_trait]
impl VoiceUpdate for NodeShard {
    async fn update_voice_state(
        &self,
        guild_id: songbird::id::GuildId,
        channel_id: Option<songbird::id::ChannelId>,
        self_deaf: bool,
        self_mute:bool
    ) -> songbird::error::JoinResult<()> {
        let payload = Fuck2 {
            guild_id: guild_id.0.to_string(),
            channel_id: channel_id.map(|channel_id| channel_id.0.to_string()),
            self_mute,
            self_deaf
        };

        let data = Fuck {
            shard_id: self.shard_id as i32,
            payload
        };

        self.submit_voice_update.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
        Ok(())
    }
}