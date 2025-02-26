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
    collections::HashMap,
    sync::{Arc, RwLock},
};

use ripple_sdk::{
    api::{
        //config::Config,
        distributor::distributor_permissions::{PermissionRequest, PermissionResponse},
        firebolt::{
            fb_capabilities::{DenyReason, DenyReasonWithCap, FireboltPermission, RoleInfo},
            fb_openrpc::CapabilitySet,
        },
        manifest::device_manifest::DeviceManifest,
    },
    framework::{file_store::FileStore, RippleResponse},
    log::info,
    utils::error::RippleError,
};

use crate::state::platform_state::PlatformState;

type FireboltPermissionStore = Arc<RwLock<FileStore<HashMap<String, Vec<FireboltPermission>>>>>;

#[derive(Debug, Clone)]
pub struct PermittedState {
    permitted: FireboltPermissionStore,
}

impl PermittedState {
    pub fn new(manifest: DeviceManifest) -> PermittedState {
        let path = get_permissions_path(manifest.configuration.saved_dir);
        let store = if let Ok(v) = FileStore::load(path.clone()) {
            v
        } else {
            FileStore::new(path, HashMap::new())
        };

        PermittedState {
            permitted: Arc::new(RwLock::new(store)),
        }
    }

    fn ingest(&mut self, extend_perms: HashMap<String, Vec<FireboltPermission>>) {
        let mut perms = self.permitted.write().unwrap();
        perms.value.extend(extend_perms);
        perms.sync();
    }

    #[cfg(test)]
    pub fn set_permissions(&mut self, permissions: HashMap<String, Vec<FireboltPermission>>) {
        let mut perms = self.permitted.write().unwrap();
        perms.value = permissions;
        perms.sync();
    }

    fn get_all_permissions(&self) -> HashMap<String, Vec<FireboltPermission>> {
        self.permitted.read().unwrap().clone().value
    }

    pub fn check_cap_role(&self, app_id: &str, role_info: RoleInfo) -> Result<bool, RippleError> {
        if let Some(role) = role_info.role {
            if let Some(perms) = self.get_all_permissions().get(app_id) {
                for perm in perms {
                    if perm.cap.as_str() == role_info.capability && perm.role == role {
                        return Ok(true);
                    }
                }
                return Ok(false);
            } else {
                // Not cached prior
                return Err(RippleError::InvalidAccess);
            }
        }
        Ok(false)
    }

    pub fn check_multiple(&self, app_id: &str, request: Vec<RoleInfo>) -> HashMap<String, bool> {
        let mut map = HashMap::new();
        for role_info in request {
            map.insert(
                role_info.clone().capability,
                if let Ok(v) = self.check_cap_role(app_id, role_info) {
                    v
                } else {
                    false
                },
            );
        }
        map
    }

    pub fn get_app_permissions(&self, app_id: &str) -> Option<Vec<FireboltPermission>> {
        if let Some(perms) = self.get_all_permissions().get(app_id) {
            return Some(perms.clone());
        }
        None
    }
}

fn get_permissions_path(saved_dir: String) -> String {
    format!("{}/{}", saved_dir, "app_perms")
}

pub struct PermissionHandler;

impl PermissionHandler {
    fn get_distributor_alias_for_app_id(ps: &PlatformState, app_id: &str) -> String {
        let dist_app_aliases = ps
            .get_device_manifest()
            .applications
            .distributor_app_aliases;
        if let Some(app_id_alias) = dist_app_aliases.get(&app_id.to_string()) {
            app_id_alias.to_string()
        } else {
            app_id.to_string()
        }
    }

    pub async fn fetch_and_store(state: &PlatformState, app_id: &str) -> RippleResponse {
        let app_id_alias = Self::get_distributor_alias_for_app_id(state, app_id);
        if state
            .cap_state
            .permitted_state
            .get_app_permissions(app_id)
            .is_some()
        {
            return Ok(());
        }
        if let Some(session) = state.session_state.get_account_session() {
            if let Ok(extn_response) = state
                .get_client()
                .send_extn_request(PermissionRequest {
                    app_id: app_id_alias,
                    session,
                })
                .await
            {
                if let Some(permission_response) = extn_response.payload.extract() {
                    let mut map = HashMap::new();
                    map.insert(app_id.to_owned(), permission_response);
                    let mut permitted_state = state.cap_state.permitted_state.clone();
                    permitted_state.ingest(map);
                    info!("Permissions fetched for {}", app_id);
                    return Ok(());
                }
            }
        }

        Err(ripple_sdk::utils::error::RippleError::InvalidOutput)
    }

    pub fn get_permitted_info(
        state: &PlatformState,
        app_id: &str,
        request: CapabilitySet,
    ) -> Result<(), DenyReasonWithCap> {
        if let Some(permitted) = state.cap_state.permitted_state.get_app_permissions(app_id) {
            let permission_set: CapabilitySet = permitted.into();
            permission_set.check(request)
        } else {
            Err(DenyReasonWithCap {
                reason: ripple_sdk::api::firebolt::fb_capabilities::DenyReason::Unpermitted,
                caps: request.get_caps(),
            })
        }
    }
    pub fn is_all_permitted(
        permitted: &[FireboltPermission],
        request: &[FireboltPermission],
    ) -> Result<(), DenyReasonWithCap> {
        let mut unpermitted: Option<FireboltPermission> = None;
        let all_permitted = request.iter().all(|perm| {
            let present = permitted.contains(perm);
            if !present {
                unpermitted = Some(perm.clone());
            }
            present
        });
        if all_permitted {
            Ok(())
        } else {
            Err(DenyReasonWithCap {
                reason: DenyReason::Unpermitted,
                caps: vec![unpermitted.unwrap().cap],
            })
        }
    }

    pub async fn get_app_permission(
        state: &PlatformState,
        app_id: &str,
    ) -> Vec<FireboltPermission> {
        let result = Vec::new();
        if let Some(permitted_caps) = state.cap_state.permitted_state.get_app_permissions(app_id) {
            return permitted_caps;
        } else if let Some(session) = state.session_state.get_account_session() {
            if let Ok(extn_response) = state
                .get_client()
                .send_extn_request(PermissionRequest {
                    app_id: app_id.to_owned(),
                    session,
                })
                .await
            {
                if let Some(permission_response) =
                    extn_response.payload.extract::<PermissionResponse>()
                {
                    return permission_response;
                }
            }
        }
        result
    }

    pub async fn check_permitted(
        state: &PlatformState,
        app_id: &str,
        request: &[FireboltPermission],
    ) -> Result<(), DenyReasonWithCap> {
        if let Some(permitted) = state.cap_state.permitted_state.get_app_permissions(app_id) {
            // return request.has_permissions(&permitted);
            return Self::is_all_permitted(&permitted, request);
        } else {
            // check to retrieve it one more time
            if (Self::fetch_and_store(state, app_id).await).is_ok() {
                // cache primed try again
                if let Some(permitted) = state.cap_state.permitted_state.get_app_permissions(app_id)
                {
                    return Self::is_all_permitted(&permitted, request);
                }
            }
        }

        Err(DenyReasonWithCap {
            reason: ripple_sdk::api::firebolt::fb_capabilities::DenyReason::Unpermitted,
            caps: request
                .iter()
                .map(|fb_perm| fb_perm.cap.to_owned())
                .collect(),
        })
    }

    pub async fn check_all_permitted(
        platform_state: &PlatformState,
        app_id: &str,
        capability: &str,
    ) -> (bool, bool, bool) {
        let mut use_granted = false;
        let mut manage_granted = false;
        let mut provide_granted = false;
        let granted_permissions = Self::get_app_permission(platform_state, app_id).await;
        for perm in granted_permissions {
            if perm.cap.as_str() == capability {
                match perm.role {
                    ripple_sdk::api::firebolt::fb_capabilities::CapabilityRole::Use => {
                        use_granted = true
                    }
                    ripple_sdk::api::firebolt::fb_capabilities::CapabilityRole::Manage => {
                        manage_granted = true
                    }
                    ripple_sdk::api::firebolt::fb_capabilities::CapabilityRole::Provide => {
                        provide_granted = true
                    }
                }
            }
        }
        (use_granted, manage_granted, provide_granted)
    }
}
