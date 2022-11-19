use std::time::Duration;
use async_trait::async_trait;
use napi::{
    JsFunction, Error, Result, Status,
    threadsafe_function::{
        ErrorStrategy,
        ThreadsafeFunction,
        ThreadsafeFunctionCallMode
    },
    bindgen_prelude::ToNapiValue
};
use napi_derive::napi;
use songbird::{
    Event, EventContext, EventHandler, TrackEvent,
    tracks::{PlayMode, ReadyState, TrackHandle}
};

use crate::to_napi_error;

#[napi(js_name = "TrackHandle")]
pub struct JsTrackHandle {
    inner: TrackHandle,
}

#[napi(js_name = "TrackHandleEvent")]
pub enum JsTrackHandleEvent {
    /// The attached track has resumed playing.
    ///
    /// This event will not fire when a track first starts,
    /// but will fire when a track changes from, e.g., paused to playing.
    /// This is most relevant for queue users: queued tracks placed into a
    /// non-empty queue are initially paused, and are later moved to `Play`.
    Play,
    /// The attached track has been paused.
    Pause,
    /// The attached track has ended.
    End,
    /// The attached track has looped.
    Loop,
    /// The attached track is being readied or recreated.
    Preparing,
    /// The attached track has become playable.
    Playable,
    /// The attached track has encountered a runtime or initialisation error.
    Error
}

impl Into<Event> for JsTrackHandleEvent {
    fn into(self) -> Event {
        let track_event = match self {
            Self::Play      => TrackEvent::Play,
            Self::Pause     => TrackEvent::Pause,
            Self::End       => TrackEvent::End,
            Self::Loop      => TrackEvent::Loop,
            Self::Preparing => TrackEvent::Preparing,
            Self::Playable  => TrackEvent::Playable,
            Self::Error     => TrackEvent::Error,
        };

        Event::Track(track_event)
    }
}

#[napi(js_name = "ReadyState")]
pub enum JsReadyState {
    /// This track hasn't been made playable yet.
    Uninitialised,
    /// The mixer is currently creating and parsing this track's byte-stream.
    Preparing,
    /// This track is fully initialised and usable.
    Playable,
}

/// Playback status of a track.
#[napi(js_name = "PlayModeValue")]
pub enum JsPlayMode {
    /// The track is currently playing.
    Play,
    /// The track is currently paused and may be resumed.
    Pause,
    /// The track has been manually stopped and cannot be restarted.
    Stop,
    /// The track has naturally ended and cannot be restarted.
    End,
    /// The track has encountered a runtime or initialisation error and cannot be restarted.
    Errored,
    /// An unknown error was encountered.
    Unknown
}

#[napi(object, js_name = "ITrackStatePlaying")]
pub struct JsTrackStatePlayingObject {
    pub value: JsPlayMode,
    pub error: Option<String>
}

#[napi(object)]
pub struct JsTrackState {
    pub position: u32,
    #[napi(ts_type = "{ value: PlayModeValue.Play }  | \
                      { value: PlayModeValue.Pause } | \
                      { value: PlayModeValue.Stop }  | \
                      { value: PlayModeValue.End }   | \
                      { value: PlayModeValue.Errored, error: string } | \
                      { value: PlayModeValue.Unknown }")]
    pub playing: JsTrackStatePlayingObject,
    pub play_time: u32,
    pub volume: f64,
    pub ready: JsReadyState,
}

impl From<PlayMode> for JsTrackStatePlayingObject {
    fn from(mode: PlayMode) -> Self {
        let mode = match mode {
            PlayMode::Play => JsPlayMode::Play,

            PlayMode::Pause => JsPlayMode::Pause,

            PlayMode::Stop => JsPlayMode::Stop,

            PlayMode::End => JsPlayMode::End,

            PlayMode::Errored(e) => return JsTrackStatePlayingObject {
                value: JsPlayMode::Errored,
                error: Some(format!("[Songbird Error] {}", e))
            },

            _ => JsPlayMode::Unknown,
        };

        JsTrackStatePlayingObject {
            value: mode,
            error: None
        }
    }
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
    pub async fn get_info(&self) -> Result<JsTrackState> {
        self.inner.get_info().await
            .map_err(to_napi_error)
            .map(|track_state| JsTrackState {
                position: track_state.position.as_millis() as u32,
                ready: match track_state.ready {
                    ReadyState::Uninitialised => JsReadyState::Uninitialised,
                    ReadyState::Preparing     => JsReadyState::Preparing,
                    ReadyState::Playable      => JsReadyState::Playable,
                },
                play_time: track_state.play_time.as_millis() as u32,
                playing: JsTrackStatePlayingObject::from(track_state.playing),
                volume: track_state.volume as f64
            })
    }

    #[napi]
    pub fn pause(&self) -> Result<()> {
        self.inner.pause().map_err(to_napi_error)
    }

    #[napi]
    pub fn resume(&self) -> Result<()> {
        self.inner.play().map_err(to_napi_error)
    }

    #[napi]
    pub fn set_volume(&self, new_volume: f64) -> Result<()> {
        self.inner.set_volume(new_volume as f32).map_err(to_napi_error)
    }

    #[napi]
    pub fn seek(&self, timecode: u32) -> Result<u32> {
        let position = Duration::from_millis(timecode as u64);
        self.inner.seek(position).result()
            .map_err(to_napi_error)
            .map(|value| value.as_millis() as u32)
    }

    #[napi]
    pub async fn seek_async(&self, timecode: u32) -> Result<u32> {
        let position = Duration::from_millis(timecode as u64);

        self.inner.seek_async(position).await
            .map_err(to_napi_error)
            .map(|value| value.as_millis() as u32)
    }

    /// Add an event listener to this track handle.
    #[napi(ts_args_type= "event: TrackHandleEvent, callback: (error: Error | null) => void")]
    pub fn add_event(&self, event: JsTrackHandleEvent, callback: JsFunction) -> Result<()> {
        let callback: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(
            0,
            |ctx| Ok(vec![ctx.value])
        ).unwrap();

        self.inner
            .add_event(event.into(), NodeEventHandler { callback })
            .map_err(to_napi_error)
    }
}

struct NodeEventHandler {
    callback: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>
}

#[async_trait]
impl EventHandler for NodeEventHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        self.callback.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);
        None
    }
}
