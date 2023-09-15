// If not stated otherwise in this file or this component's LICENSE file the
// following copyright and licenses apply:
//
// Copyright 2023 Liberty Global Service B.V.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use ripple_sdk::{
    api::device::{
        device_peristence::{DevicePersistenceRequest, GetStorageProperty, SetStorageProperty, StorageData},
    },
    extn::{
        client::{
            extn_client::ExtnClient,
            extn_processor::{
                DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
            },
        },
        extn_client_message::{ExtnMessage, ExtnResponse},
    },
    serde::{Deserialize, Serialize},
    async_trait::async_trait,
    utils::error::RippleError,
    log::error,
};

use tungstenite::{connect, Message};

#[derive(Debug)]
pub struct PersistentStorageRequestProcessor {
    client: ExtnClient,
    streamer: DefaultExtnStreamer,
}

#[derive(Serialize, Deserialize)]
struct IApplicationServicesResponce {
    result: String,
}

pub fn extract_value(input: &str) -> String {
    // responce from IApplicationServicesResponce is in fromat <method/name>:<correct JSON>
    let index = input.find(":");
    let (_first, last) = input.split_at(index.unwrap()+1);
    let value: IApplicationServicesResponce = serde_json::from_str(last).expect("JSON was not well-formatted");
    return value.result;
}

pub async fn retrive_application_services_value(token: &str) -> String {

    let addr = "ws://127.0.0.1:10415";
    let command = format!(r#"settings/getSetting:{{"payload":"{}"}}"#, token);
    let (mut socket, _response) = connect(addr).unwrap();
    error!("Connected to the server");

    socket.send(Message::Text(command.into())).unwrap();
    let msg = socket.read().expect("Error reading message");
    error!("Received: {}", msg);

    return extract_value(msg.into_text().unwrap().as_str());
}   

impl PersistentStorageRequestProcessor {

    pub fn new(client: ExtnClient) -> PersistentStorageRequestProcessor {
        PersistentStorageRequestProcessor {
            client,
            streamer: DefaultExtnStreamer::new(),
        }
    }

    async fn get_value(client: ExtnClient, req: ExtnMessage, data: GetStorageProperty) -> bool {
        match data.namespace.as_str() {
            "DeviceName" => {
                match data.key.as_str() {
                    "name" => {
                        let value = retrive_application_services_value("cpe.friendlyName").await;
                        let result = StorageData::new(serde_json::Value::String(value));
                        return Self::respond(
                            client,
                            req.clone(),
                            ExtnResponse::StorageData(result),
                        )
                        .await
                        .is_ok();
                    }
                    _ => return Self::handle_error(client, req, RippleError::ProcessorError).await
                }
            }
            _ => return Self::handle_error(client, req, RippleError::ProcessorError).await
        }
    }

    async fn set_value(client: ExtnClient, req: ExtnMessage, _data: SetStorageProperty) -> bool {
        Self::handle_error(client, req, RippleError::ProcessorError).await
    }
}

impl ExtnStreamProcessor for PersistentStorageRequestProcessor {
    type STATE = ExtnClient;
    type VALUE = DevicePersistenceRequest;

    fn get_state(&self) -> Self::STATE {
        self.client.clone()
    }

    fn receiver(&mut self) -> ripple_sdk::tokio::sync::mpsc::Receiver<ExtnMessage> {
        self.streamer.receiver()
    }

    fn sender(&self) -> ripple_sdk::tokio::sync::mpsc::Sender<ExtnMessage> {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for PersistentStorageRequestProcessor {
    fn get_client(&self) -> ExtnClient {
        self.client.clone()
    }
    async fn process_request(
        state: Self::STATE,
        msg: ripple_sdk::extn::extn_client_message::ExtnMessage,
        extracted_message: Self::VALUE,
    ) -> bool {
        match extracted_message {
            DevicePersistenceRequest::Get(get_params) => {
                Self::get_value(state.clone(), msg, get_params).await
            }
            DevicePersistenceRequest::Set(set_params) => {
                Self::set_value(state.clone(), msg, set_params).await
            }
        }
    }
}