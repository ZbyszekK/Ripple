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
    api::{
        device::device_peristence::{DevicePersistenceRequest, GetStorageProperty, SetStorageProperty, StorageData},
        storage_property::{
            NAMESPACE_CLOSED_CAPTIONS, NAMESPACE_DEVICE_NAME, NAMESPACE_LOCALIZATION,
            KEY_ENABLED, KEY_NAME, KEY_LANGUAGE, KEY_COUNTRY_CODE, KEY_LOCALE,
        },
        storage_events::{
            StorageEvent,
            StorageEventRequest,
        },
        device::device_request::VoiceGuidanceState,
    },
    extn::{
        client::{
            extn_client::ExtnClient,
            extn_processor::{
                DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
            },
        },
        extn_client_message::{
            ExtnEvent,
            ExtnMessage,
            ExtnResponse,
        },
    },
    serde::{Deserialize, Serialize},
    async_trait::async_trait,
    utils::time_utils::Timer,
    utils::error::RippleError,
    log::{error, info},
};

use tungstenite::{
    connect, Message, WebSocket,
    stream::MaybeTlsStream,
};

use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};

type Wss = WebSocket<MaybeTlsStream<TcpStream>>;

pub const CONNECTION_STRING: &'static str = "ws://127.0.0.1:10415";

pub const PROFILE_SUBTITLES: &'static str = r#"settings/getSetting:{"payload":"profile.subControl"}"#;
pub const PROFILE_OSD_LANG:  &'static str = r#"settings/getSetting:{"payload":"profile.osdLang"}"#;
pub const CPE_FRIENDLY_NAME: &'static str = r#"settings/getSetting:{"payload":"cpe.friendlyName"}"#;
pub const CPE_COUNTRY:       &'static str = r#"configuration/getConfig:{"payload":"cpe.country"}"#;


#[derive(Debug)]
pub struct PersistentStorageRequestProcessor {
    state: ApplicationServicesState,
    streamer: DefaultExtnStreamer,
}

#[derive(Debug, Clone)]
pub struct ApplicationServicesState {
    client: ExtnClient,
    /*TODO: create separate ApplicationServices client with thread and shared web seocket
     *      this Arc<RwLock<Wss>> is only for RW access to scocket from cloned State
     */
    socket: Arc<RwLock<Wss>>,
    timer: Arc<RwLock<Option<Timer>>>,
}

impl ApplicationServicesState {
    fn new(client: ExtnClient) -> Self {

        let (socket, _response) =
            connect(CONNECTION_STRING).unwrap();

        Self {
            client,
            socket: Arc::new(RwLock::new(socket)),
            timer: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ApplicationServicesResponceType {
    BOOLEAN,
    STRING,
    INTEGER,
    NONE,
}

pub fn expected_type(token: &str) -> ApplicationServicesResponceType {
    match token {
        PROFILE_SUBTITLES => ApplicationServicesResponceType::BOOLEAN,
        PROFILE_OSD_LANG  => ApplicationServicesResponceType::STRING,
        CPE_FRIENDLY_NAME => ApplicationServicesResponceType::STRING,
        CPE_COUNTRY       => ApplicationServicesResponceType::STRING,
        _ => ApplicationServicesResponceType::NONE,
    }
}

pub fn translate_namespace_and_key(namespace: &str, key: &str) -> Option<&'static str> {
    match namespace {
        NAMESPACE_CLOSED_CAPTIONS => {
            match key {
                KEY_ENABLED => Some(PROFILE_SUBTITLES),
                _ => None,
            }
        }
        NAMESPACE_DEVICE_NAME => {
            match key {
                KEY_NAME => Some(CPE_FRIENDLY_NAME),
                _ => None,
            }
        }
        NAMESPACE_LOCALIZATION => {
            match key {
                KEY_LANGUAGE => Some(PROFILE_OSD_LANG),
                KEY_COUNTRY_CODE => Some(CPE_COUNTRY),
                KEY_LOCALE => Some(PROFILE_OSD_LANG),
                _ => None,
            }
        }
        _ => None,
    }
}


/* TODO: Change those 2 methods to template */
 
 #[derive(Serialize, Deserialize)]
 struct IApplicationServicesBooleanResponce {
     result: bool,
 }
 
 #[derive(Serialize, Deserialize)]
 struct IApplicationServicesStringResponce {
     result: String,
 }

pub fn extract_bool_value(input: &str) -> Option<bool> {
    match serde_json::from_str::<IApplicationServicesBooleanResponce>(&input) {
        Ok(al) => Some(al.result),
        Err(_) => {
            error!("Parsing AplicationServices response {} was unsuccessful", input);
            None
        }
    }
}

pub fn extract_string_value(input: &str) -> Option<String> {
    match serde_json::from_str::<IApplicationServicesStringResponce>(&input) {
        Ok(al) => Some(al.result),
        Err(_) => {
            error!("Parsing AplicationServices response {} was unsuccessful", input);
            None
        }
    }
}

/* TODO: async code instead synchronous socket send/read */
/* TODO: error handling */
pub async fn retrive_application_services_value(socket_ref: Arc<RwLock<Wss>>, token: &str) -> StorageData {
    let mut socket = socket_ref.write().unwrap();
    let _res = socket.send(Message::Text(token.into()));
    let msg = socket.read().expect("Error reading message");
    let response = msg.into_text().unwrap();
    let index = response.find(":");
    let (_first, last) = response.split_at(index.unwrap()+1);
    info!("Parsed ApplicationServices response {:?}", last);
    match expected_type(token) {
        ApplicationServicesResponceType::BOOLEAN => {
            return StorageData::new(serde_json::Value::Bool(extract_bool_value(last).unwrap()));
        }
        ApplicationServicesResponceType::STRING => {
            return StorageData::new(serde_json::Value::String(extract_string_value(last).unwrap()));
        }
        _ => {
            return StorageData::new(serde_json::Value::Null);
        }
    }
}

impl PersistentStorageRequestProcessor {

    pub fn new(client: ExtnClient) -> PersistentStorageRequestProcessor {
        PersistentStorageRequestProcessor {
            state: ApplicationServicesState::new(client),
            streamer: DefaultExtnStreamer::new(),
        }
    }

    async fn get_value(state: ApplicationServicesState, req: ExtnMessage, data: GetStorageProperty) -> bool {
        if let Some(as_key) = translate_namespace_and_key(data.namespace.as_str(), data.key.as_str()) {
            let value = retrive_application_services_value(state.socket, as_key).await;
            return Self::respond(
                state.client,
                req.clone(),
                ExtnResponse::StorageData(value),
            )
            .await
            .is_ok();
        }
        Self::handle_error(state.client, req, RippleError::ProcessorError).await
    }


    async fn set_value(state: ApplicationServicesState, req: ExtnMessage, _data: SetStorageProperty) -> bool {
        Self::handle_error(state.client, req, RippleError::ProcessorError).await
    }
}

impl ExtnStreamProcessor for PersistentStorageRequestProcessor {
    type STATE = ApplicationServicesState;
    type VALUE = DevicePersistenceRequest;

    fn get_state(&self) -> Self::STATE {
        self.state.clone()
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
        self.state.clone().client
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


// TODO real websocket client that register for AS notifications
// TODO this fake event stuck on ripple core for "internal" target
pub async fn generate_fake_event(mut state: ApplicationServicesState) {

    //Storage event tryout
    //let value = serde_json::to_value("").unwrap();
    //let result = serde_json::to_value("false").unwrap();
    //let event = ExtnEvent::AppEvent(AppEventRequest::Emit(AppEvent {
    //    event_name: EVENT_CLOSED_CAPTIONS_SETTINGS_CHANGED.to_string(),
    //    context: None,
    //    result,
    //    app_id: None,
    //}));

    //Let use instead some well known event instead starage event for testing
    let event = ExtnEvent::VoiceGuidanceState(
        VoiceGuidanceState {
            state: false,
        }
    );

    //TODO is this right method to pass events to Ripple core ?
    info!("Sending fake event with request_transient() ... ");
    let rr = state.client.request_transient(event);
    if rr.is_err() {
        error!("Error while forwarding cc change event");
    } else {
        info!("Sended !");
    }
}


#[derive(Debug)]
pub struct PersistentStoreEventProcessor {
    state: ApplicationServicesState,
    streamer: DefaultExtnStreamer,
}

impl PersistentStoreEventProcessor {
    pub fn new(client: ExtnClient) -> PersistentStoreEventProcessor {
        PersistentStoreEventProcessor {
            state: ApplicationServicesState::new(client),
            streamer: DefaultExtnStreamer::new(),
        }
    }
}

impl ExtnStreamProcessor for PersistentStoreEventProcessor {
    type STATE = ApplicationServicesState;
    type VALUE = StorageEventRequest;

    fn get_state(&self) -> Self::STATE {
        self.state.clone()
    }

    fn receiver(&mut self) -> ripple_sdk::tokio::sync::mpsc::Receiver<ExtnMessage> {
        self.streamer.receiver()
    }

    fn sender(&self) -> ripple_sdk::tokio::sync::mpsc::Sender<ExtnMessage> {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for PersistentStoreEventProcessor {
    fn get_client(&self) -> ExtnClient {
        self.state.clone().client
    }

    async fn process_request(
        state: Self::STATE,
        msg: ExtnMessage,
        extracted_message: Self::VALUE,
    ) -> bool {
   
        let event = extracted_message.clone().event;

        match event {
            StorageEvent::ClosedCaptionsEnabledChanged => {
                info!("ClosedCaptionsEnabledChanged event");
                //Lets generate fake CaptionsEnabledChanged event in 5 second
                *state.timer.write().unwrap() = Some(Timer::start(3000, generate_fake_event(state.clone())));
            },
            StorageEvent::DeviceNameChanged => {
                info!("DeviceNameChanged event");
                //Lets generate fake DeviceNameChanged event in 5 second
                *state.timer.write().unwrap() = Some(Timer::start(3000, generate_fake_event(state.clone())));
            },
        };

        Self::ack(state.clone().client, msg).await.is_ok()
    }
}

