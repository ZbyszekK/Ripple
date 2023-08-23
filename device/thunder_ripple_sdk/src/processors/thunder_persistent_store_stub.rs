// Copyright 2023 Comcast Cable Communications Management, LLC
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
// SPDX-License-Identifier: Apache-2.0
//

use crate::{
    ripple_sdk::{
        api::device::{
            device_peristence::{DevicePersistenceRequest, GetStorageProperty, SetStorageProperty},
        },
        async_trait::async_trait,
        extn::{
            client::extn_client::ExtnClient,
            client::extn_processor::{
                DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
            },
            extn_client_message::{ExtnMessage, ExtnResponse},
        },
        log::{info},
        serde_json::{self, Value},
        tokio::sync::mpsc,
    },
    thunder_state::ThunderState,
};

#[derive(Debug)]
pub struct ThunderStorageRequestProcessor {
    state: ThunderState,
    streamer: DefaultExtnStreamer,
}

impl ThunderStorageRequestProcessor {
    pub fn new(state: ThunderState) -> ThunderStorageRequestProcessor {
        ThunderStorageRequestProcessor {
            state,
            streamer: DefaultExtnStreamer::new(),
        }
    }

    #[allow(dead_code)]
    async fn delete_key(self, namespace: String, key: String) -> bool {
        let merged_key: String = format!("{}.{}", namespace, key);
        info!("ZK delete_key : {:?}", merged_key);
        true
    }

    #[allow(dead_code)]
    async fn delete_namespace(self, namespace: String) -> bool {
        info!("ZK delete_namespace : {:?}", namespace);
        true
    }

    #[allow(dead_code)]
    async fn flush_cache(self) -> bool {
        info!("ZK flush_cache");
        true
    }

    async fn get_value(state: ThunderState, req: ExtnMessage, data: GetStorageProperty) -> bool {
        let merged_key: String = format!("{}.{}", data.namespace, data.key);
        info!("ZK get_value : {:?}", merged_key);
        match merged_key.as_str() {
            "Localization.language" => {
                let resp = r#"{"value":"En"}"#;
                let parsed_res: Result<Value, serde_json::Error> = serde_json::from_str(resp);
                println!("{:#?}", parsed_res);
                let value = parsed_res.unwrap();
                if let Ok(v) = serde_json::from_value(value.clone()) {
                    println!("will return ...");
                    return Self::respond(
                        state.get_client(),
                        req.clone(),
                        ExtnResponse::StorageData(v),
                    )
                    .await
                    .is_ok();
                }
                return Self::respond(state.get_client(), req.clone(), ExtnResponse::None(()))
                .await
                .is_ok();
            },
            _ => {
                println!("ZK get_value {} - unknown key", merged_key);
                return Self::respond(state.get_client(), req.clone(), ExtnResponse::None(()))
                .await
                .is_ok();
            }
        }
    }

    async fn set_value(_state: ThunderState, _req: ExtnMessage, data: SetStorageProperty) -> bool {
        let merged_key: String = format!("{}.{}", data.namespace, data.key);
        info!("ZK set_value : {:?}", merged_key);
        return true;
    }
}


impl ExtnStreamProcessor for ThunderStorageRequestProcessor {
    type STATE = ThunderState;
    type VALUE = DevicePersistenceRequest;

    fn get_state(&self) -> Self::STATE {
        self.state.clone()
    }

    fn receiver(&mut self) -> mpsc::Receiver<ExtnMessage> {
        self.streamer.receiver()
    }

    fn sender(&self) -> mpsc::Sender<ExtnMessage> {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for ThunderStorageRequestProcessor {
    fn get_client(&self) -> ExtnClient {
        self.state.get_client()
    }
    async fn process_request(
        state: Self::STATE,
        msg: ExtnMessage,
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
