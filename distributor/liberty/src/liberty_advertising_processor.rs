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
    serde::{Deserialize, Serialize},
    api::firebolt::fb_advertising::{
        AdIdResponse, AdInitObjectResponse, AdvertisingRequest, AdvertisingResponse,
    },
    async_trait::async_trait,
    extn::{
        client::{
            extn_client::ExtnClient,
            extn_processor::{
                DefaultExtnStreamer, ExtnRequestProcessor, ExtnStreamProcessor, ExtnStreamer,
            },
        },
        extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnResponse},
    },
    log::error,
};

use tungstenite::{connect, Message};

pub struct DistributorAdvertisingProcessor {
    client: ExtnClient,
    streamer: DefaultExtnStreamer,
}

#[derive(Serialize, Deserialize)]
struct IfaceFResponce {
    result: String,
}

pub fn extract_cpeid(input: &str) -> String {
    // responce from IfaceF is in fromat <method/name>:<correct JSON>
    let index = input.find(":");
    let (_first, last) = input.split_at(index.unwrap()+1);
    let value: IfaceFResponce = serde_json::from_str(last).expect("JSON was not well-formatted");
    return value.result;
}

impl DistributorAdvertisingProcessor {
    pub fn new(client: ExtnClient) -> DistributorAdvertisingProcessor {
        DistributorAdvertisingProcessor {
            client,
            streamer: DefaultExtnStreamer::new(),
        }
    }

    pub async fn retrive_cpeid() -> String {

        let addr = "ws://127.0.0.1:10415";
        let command = r#"configuration/getConfig:{"payload":"cpe.id"}"#;
        
        let (mut socket, _response) = connect(addr).unwrap();
        error!("Connected to the server");

        socket.send(Message::Text(command.into())).unwrap();
        let msg = socket.read().expect("Error reading message");
        error!("Received: {}", msg);

        return extract_cpeid(msg.into_text().unwrap().as_str());
	}     
}

impl ExtnStreamProcessor for DistributorAdvertisingProcessor {
    type STATE = ExtnClient;
    type VALUE = AdvertisingRequest;

    fn get_state(&self) -> Self::STATE {
        self.client.clone()
    }

    fn receiver(
        &mut self,
    ) -> ripple_sdk::tokio::sync::mpsc::Receiver<ripple_sdk::extn::extn_client_message::ExtnMessage>
    {
        self.streamer.receiver()
    }

    fn sender(
        &self,
    ) -> ripple_sdk::tokio::sync::mpsc::Sender<ripple_sdk::extn::extn_client_message::ExtnMessage>
    {
        self.streamer.sender()
    }
}

#[async_trait]
impl ExtnRequestProcessor for DistributorAdvertisingProcessor {
    fn get_client(&self) -> ExtnClient {
        self.client.clone()
    }
    async fn process_request(
        state: Self::STATE,
        msg: ripple_sdk::extn::extn_client_message::ExtnMessage,
        extracted_message: Self::VALUE,
    ) -> bool {
        match extracted_message {
            AdvertisingRequest::ResetAdIdentifier(_session) => {
                error!("ZK ResetAdIdentifier Liberty");
                if let Err(e) = state
                    .clone()
                    .respond(
                        msg,
                        ripple_sdk::extn::extn_client_message::ExtnResponse::None(()),
                    )
                    .await
                {
                    error!("Error sending back response {:?}", e);
                    return false;
                }
                true
            }
            AdvertisingRequest::GetAdInitObject(_as_init_obj) => {

                // Currently no IfA defined for apollo V1+, let re use this method to test communication and 
                // obtain some values from DBus 
                error!("ZK Liberty - no Ifa implementation");

                let resp = AdInitObjectResponse {
                    ad_server_url: "".into(),
                    ad_server_url_template: "".into(),
                    ad_network_id: "".into(),
                    ad_profile_id: "".into(),
                    ad_site_section_id: "".into(),
                    ad_opt_out: true,
                    // Mock invalidated token for schema validation
                    privacy_data: "".into(),
                    ifa_value: "".into(),
                    // Mock invalidated token for schema validation
                    ifa: "".into(),
                    app_name: "".into(),
                    app_version: "".into(),
                    app_bundle_id: "".into(),
                    distributor_app_id: "".into(),
                    device_ad_attributes: "".into(),
                    coppa: 0.to_string(),
                    authentication_entity: "".into(),
                };

                if let Err(e) = state
                    .clone()
                    .respond(
                        msg,
                        if let ExtnPayload::Response(r) =
                            AdvertisingResponse::AdInitObject(Box::new(resp)).get_extn_payload()
                        {
                            r
                        } else {
                            ExtnResponse::Error(
                                ripple_sdk::utils::error::RippleError::ProcessorError,
                            )
                        },
                    )
                    .await
                {
                    error!("Error sending back response {:?}", e);
                    return false;
                }
                true
            }

            AdvertisingRequest::GetAdIdObject(_ad_id_req) => {

                // Currently na IfA defined for apollo V1+, let re use this method to test communication and 
                // obtain some values from IfaceF with web sockets

                error!("Liberty - no Ifa implementation, this is WebSocket Iface F test connection method");

                let myifa = DistributorAdvertisingProcessor::retrive_cpeid().await;

                let resp = AdIdResponse {
                    ifa: myifa.into(),
                    ifa_type: "".into(),
                    lmt: "0".into(),
                };
                if let Err(e) = state
                    .clone()
                    .respond(
                        msg,
                        if let ExtnPayload::Response(r) =
                            AdvertisingResponse::AdIdObject(resp).get_extn_payload()
                        {
                            r
                        } else {
                            ExtnResponse::Error(
                                ripple_sdk::utils::error::RippleError::ProcessorError,
                            )
                        },
                    )
                    .await
                {
                    error!("Error sending back response {:?}", e);
                    return false;
                }
                true
            }
        }
    }
}
