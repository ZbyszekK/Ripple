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

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use ripple_sdk::{
    api::{
        firebolt::fb_capabilities::{
            DenyReason, DenyReasonWithCap, FireboltCap, FireboltPermission,
        },
        manifest::device_manifest::DeviceManifest,
    },
    log::debug,
};

#[derive(Clone, Debug, Default)]
pub struct GenericCapState {
    supported: Arc<RwLock<HashSet<String>>>,
    // it consumes less memory and operations to store not_available vs available
    not_available: Arc<RwLock<HashSet<String>>>,
}

impl GenericCapState {
    pub fn new(manifest: DeviceManifest) -> GenericCapState {
        let cap_state = GenericCapState::default();
        cap_state.ingest_supported(manifest.get_supported_caps());
        cap_state
    }

    pub fn ingest_supported(&self, request: Vec<FireboltCap>) {
        let mut supported = self.supported.write().unwrap();
        supported.extend(
            request
                .iter()
                .map(|a| a.as_str())
                .collect::<HashSet<String>>(),
        )
    }

    pub fn ingest_availability(&self, request: Vec<FireboltCap>, is_available: bool) {
        let mut not_available = self.not_available.write().unwrap();
        for cap in request {
            if is_available {
                not_available.remove(&cap.as_str());
            } else {
                not_available.insert(cap.as_str());
            }
        }
    }

    pub fn check_for_processor(&self, request: Vec<String>) -> HashMap<String, bool> {
        let supported = self.supported.read().unwrap();
        let mut result = HashMap::new();
        for cap in request {
            result.insert(cap.clone(), supported.contains(&cap));
        }
        result
    }

    pub fn check_supported(&self, request: &[FireboltPermission]) -> Result<(), DenyReasonWithCap> {
        let supported = self.supported.read().unwrap();
        let not_supported: Vec<FireboltCap> = request
            .iter()
            .filter(|fb_perm| !supported.contains(&fb_perm.cap.as_str()))
            .map(|fb_perm| fb_perm.cap.clone())
            .collect();

        debug!(
            "checking supported caps request={:?}, not_supported={:?}",
            request, not_supported
        );

        if !not_supported.is_empty() {
            return Err(DenyReasonWithCap::new(
                DenyReason::Unsupported,
                not_supported,
            ));
        }
        Ok(())
    }

    pub fn check_available(
        &self,
        request: &Vec<FireboltPermission>,
    ) -> Result<(), DenyReasonWithCap> {
        let not_available = self.not_available.read().unwrap();
        let mut result: Vec<FireboltCap> = Vec::new();
        for fb_perm in request {
            if not_available.contains(&fb_perm.cap.as_str()) {
                result.push(fb_perm.cap.clone())
            }
        }
        if !result.is_empty() {
            return Err(DenyReasonWithCap::new(DenyReason::Unavailable, result));
        }
        Ok(())
    }

    pub fn check_all(
        &self,
        permissions: &Vec<FireboltPermission>,
    ) -> Result<(), DenyReasonWithCap> {
        self.check_supported(permissions)?;
        self.check_available(permissions)
    }
}
