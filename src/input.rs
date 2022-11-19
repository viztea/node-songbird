use napi::{Result, Error};
use napi_derive::napi;
use reqwest::Client;
use songbird::input::{Input, HttpRequest, YoutubeDl};

enum InputType {
    YouTube,
    HTTP,
    File,
}

#[napi(js_name = "Input")]
pub struct JsInput {
    input_type: InputType,
    http_client: Option<ReqwestClient>,
    pub identifier: String
}

#[napi]
#[derive(Clone)]
pub struct ReqwestClient {
    pub(crate) inner: Client
}

#[napi]
pub struct JsAuxMetadata {
    /// The track name of this stream.
    pub track: Option<String>,
    /// The main artist of this stream.
    pub artist: Option<String>,
    /// The album name of this stream.
    pub album: Option<String>,
    /// The date of creation of this stream.
    pub date: Option<String>,

    /// The number of audio channels in this stream.
    pub channels: Option<u8>,
    /// The YouTube channel of this stream.
    pub channel: Option<String>,
    /// The time at which the first true sample is played back.
    ///
    /// This occurs as an artefact of coder delay.
    pub start_time: Option<u32>,
    /// The reported duration of this stream.
    pub duration: Option<u32>,
    /// The sample rate of this stream.
    pub sample_rate: Option<u32>,
    /// The source url of this stream.
    pub source_url: Option<String>,
    /// The YouTube title of this stream.
    pub title: Option<String>,
    /// The thumbnail url of this stream.
    pub thumbnail: Option<String>,
}

#[napi]
impl ReqwestClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: Client::new() }
    }
}

#[napi]
impl JsInput {
    #[napi(factory)]
    pub fn youtube(http_client: &ReqwestClient, identifier: String) -> Self {
        Self { input_type: InputType::YouTube, identifier, http_client: Some(http_client.clone()) }
    }

    #[napi(factory)]
    pub fn http(http_client: &ReqwestClient, url: String/*, headers: JsObject*/) -> Self {
        Self { input_type: InputType::HTTP, identifier: url, http_client: Some(http_client.clone()) }
    }

    #[napi(factory)]
    pub fn file(path: String) -> Self {
        Self { input_type: InputType::File, identifier: path, http_client: None }
    }

    #[napi]
    pub async fn get_aux_metadata(&self) -> Result<JsAuxMetadata> {
        self.create_input()
            .aux_metadata()
            .await
            .map(|metadata| JsAuxMetadata {
                track: metadata.track,
                artist: metadata.artist,
                album: metadata.album,
                date: metadata.date,
                channels: metadata.channels,
                channel: metadata.channel,
                start_time: metadata.start_time.map(|st| st.as_millis() as u32),
                duration: metadata.duration.map(|st| st.as_millis() as u32),
                sample_rate: metadata.sample_rate,
                source_url: metadata.source_url,
                title: metadata.title,
                thumbnail: metadata.thumbnail
            })
            .map_err(|error| Error::from_reason(format!("{}", error)))
    }

    pub(crate) fn create_input(&self) -> Input {
        match self.input_type {
            InputType::YouTube => {
                let http_client = self.http_client.as_ref().unwrap().clone().inner;
                YoutubeDl::new(http_client, self.identifier.to_owned()).into()
            },
            InputType::HTTP => {
                let http_client = self.http_client.as_ref().unwrap().clone().inner;
                HttpRequest::new(http_client, self.identifier.to_owned()).into()
            },
            InputType::File => {
                panic!("not implemented")
            },
        }
    }
}
