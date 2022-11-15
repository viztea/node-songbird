use std::sync::Arc;
use napi_derive::napi;
use songbird::input::{Input, HttpRequest, YoutubeDl};
use crate::manager::Manager;

enum InputType {
    YouTube,
    HTTP,
    File,
}

#[napi(js_name = "Input")]
pub struct JsInput {
    input_type: InputType,
    manager: Option<Arc<Manager>>,
    pub identifier: String
}

#[napi]
impl JsInput {
    #[napi(factory)]
    pub fn youtube(manager: &Manager, identifier: String) -> Self {
        Self { input_type: InputType::YouTube, identifier, manager: Some(Arc::new(manager.clone())) }
    }

    #[napi(factory)]
    pub fn http(manager: &Manager, url: String) -> Self {
        Self { input_type: InputType::HTTP, identifier: url, manager: Some(Arc::new(manager.clone())) }
    }

    #[napi(factory)]
    pub fn file(path: String) -> Self {
        Self { input_type: InputType::File, identifier: path, manager: None }
    }

    pub(crate) fn create_input(&self) -> Input {
        match self.input_type {
            InputType::YouTube => {
                let http_client = &self.manager.as_ref().unwrap().http_client; // TODO(gino) cleanup
                YoutubeDl::new(http_client.clone(), self.identifier.to_owned()).into()
            },
            InputType::HTTP => {
                let http_client = &self.manager.as_ref().unwrap().http_client; // TODO(gino) cleanup
                HttpRequest::new(http_client.clone(), self.identifier.to_owned()).into()
            },
            InputType::File => {
                panic!("not implemented")
            },
        }
    }
}
