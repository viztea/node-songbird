use std::sync::Arc;
use crate::driver::shards::NodeSharder;

pub mod shards;

pub(crate) struct Driver {
    pub(crate) sharder: Arc<NodeSharder>,
    pub(crate) shard_count: u64,
}
