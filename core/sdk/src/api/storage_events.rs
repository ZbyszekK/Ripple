use crate::{
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest},
    framework::ripple_contract::RippleContract,
};
use log::{error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEvent {
    ClosedCaptionsEnabledChanged,
    DeviceNameChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEventRequest {
    pub event: StorageEvent,
    pub subscribe: bool,
    pub id: String,
}

impl ExtnPayloadProvider for StorageEventRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        error!("ZK ExtnPayloadProvider get_extn_payload");
        ExtnPayload::Request(ExtnRequest::StorageEvent(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        if let ExtnPayload::Request(ExtnRequest::StorageEvent(d)) = payload {
            error!("ZK ExtnPayloadProvider get_from_payload");
            return Some(d);
        }

        None
    }

    fn contract() -> RippleContract {
        RippleContract::StorageEvents
    }
}