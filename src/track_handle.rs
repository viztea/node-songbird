use std::time::Duration;
use async_trait::async_trait;
use napi::{Error, error, JsFunction, Result, Status};
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use songbird::{Event, EventContext, EventHandler, TrackEvent};
use songbird::tracks::TrackHandle;

#[napi(js_name = "TrackHandle")]
pub struct JsTrackHandle {
    inner: TrackHandle,
}

#[napi]
impl JsTrackHandle {
    #[napi(factory)]
    pub fn create() -> Result<Self> {
        Err(Error::from_status(Status::Unknown))
    }

    pub(crate) fn new(inner: TrackHandle) -> Self {
        Self { inner }
    }

    #[napi]
    pub fn pause(&self) {
        self.inner.pause().unwrap()
    }

    #[napi]
    pub fn resume(&self) {
         self.inner.play().unwrap()
    }

    #[napi]
    pub async fn seek(&self, timecode: u32) -> u32 {
        let position = Duration::from_millis(timecode as u64);

        self.inner.seek_async(position).await.unwrap().subsec_millis()
    }

    #[napi(ts_args_type= "event: 'play' | 'pause' | 'end' | 'error', callback: (error: Error | null) => void")]
    pub fn add_event(&self, event: String, callback: JsFunction) {
        let event = Event::Track(match event.to_lowercase().as_str() {
            "play"  => TrackEvent::Play,
            "pause" => TrackEvent::Pause,
            "end"   => TrackEvent::End,
            "error" => TrackEvent::Error,
            _ => return// Err(Error::from_reason("Invalid Event"))
        });

        let callback: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(
            0,
            |ctx| Ok(vec![ctx.value])
        ).unwrap();

        self.inner.add_event(event, NodeEventHandler { callback }).unwrap();
    }
}

struct NodeEventHandler {
    callback: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>
}

#[async_trait]
impl EventHandler for NodeEventHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        self.callback.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);
        None
    }
}
