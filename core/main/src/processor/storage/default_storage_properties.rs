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

use ripple_sdk::{
    api::storage_property::{
        KEY_BACKGROUND_COLOR, KEY_BACKGROUND_OPACITY, KEY_COUNTRY_CODE, KEY_ENABLED,
        KEY_FONT_COLOR, KEY_FONT_EDGE, KEY_FONT_EDGE_COLOR, KEY_FONT_FAMILY, KEY_FONT_OPACITY,
        KEY_FONT_SIZE, KEY_LANGUAGE, KEY_LOCALE, KEY_NAME, KEY_SKIP_RESTRICTION, KEY_TEXT_ALIGN,
        KEY_TEXT_ALIGN_VERTICAL, NAMESPACE_ADVERTISING, NAMESPACE_CLOSED_CAPTIONS,
        NAMESPACE_DEVICE_NAME, NAMESPACE_LOCALIZATION,
    },
    log::debug,
};

use crate::state::platform_state::PlatformState;

#[derive(Debug, Clone)]
pub enum DefaultStoragePropertiesError {
    UnreconizedKey(String),
    UnreconizedNamespace(String),
}

#[derive(Clone, Debug)]
pub struct DefaultStorageProperties;

impl DefaultStorageProperties {
    pub fn get_bool(
        state: &PlatformState,
        namespace: &String,
        key: &'static str,
    ) -> Result<bool, DefaultStoragePropertiesError> {
        debug!("get_bool: namespace={}, key={}", namespace, key);
        if namespace.eq(NAMESPACE_CLOSED_CAPTIONS) {
            match key {
                KEY_ENABLED => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .enabled),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else {
            Err(DefaultStoragePropertiesError::UnreconizedNamespace(
                namespace.to_owned(),
            ))
        }
    }

    pub fn get_string(
        state: &PlatformState,
        namespace: &String,
        key: &'static str,
    ) -> Result<String, DefaultStoragePropertiesError> {
        debug!("get_string: namespace={}, key={}", namespace, key);
        if namespace.eq(NAMESPACE_CLOSED_CAPTIONS) {
            match key {
                KEY_FONT_FAMILY => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_family),
                KEY_FONT_COLOR => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_color),
                KEY_FONT_EDGE => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_edge),
                KEY_FONT_EDGE_COLOR => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_edge_color),
                KEY_BACKGROUND_COLOR => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .background_color),
                KEY_TEXT_ALIGN => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .text_align),
                KEY_TEXT_ALIGN_VERTICAL => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .text_align_vertical),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else if namespace.eq(NAMESPACE_DEVICE_NAME) {
            match key {
                KEY_NAME => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .name),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else if namespace.eq(NAMESPACE_LOCALIZATION) {
            match key {
                KEY_COUNTRY_CODE => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .country_code),
                KEY_LANGUAGE => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .language),
                KEY_LOCALE => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .locale),
                // Not used anywhere just yet
                // KEY_ADDITIONAL_INFO => {
                //     let a_info_map: HashMap<String, String> = state.get_device_manifest().clone().configuration.default_values.additional_info;
                //     Ok(serde_json::to_string(&a_info_map).unwrap())
                // }
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else if let Some(defaults) = state
            .get_device_manifest()
            .get_settings_defaults_per_app()
            .get(namespace)
        {
            Ok(defaults.postal_code.clone())
        } else if namespace.eq(NAMESPACE_ADVERTISING) {
            match key {
                KEY_SKIP_RESTRICTION => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .skip_restriction),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else {
            Err(DefaultStoragePropertiesError::UnreconizedNamespace(
                namespace.to_owned(),
            ))
        }
    }

    pub fn get_number_as_u32(
        state: &PlatformState,
        namespace: &String,
        key: &'static str,
    ) -> Result<u32, DefaultStoragePropertiesError> {
        debug!("get_number_as_u32: namespace={}, key={}", namespace, key);
        if namespace.eq(NAMESPACE_CLOSED_CAPTIONS) {
            match key {
                KEY_FONT_OPACITY => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_opacity),
                KEY_BACKGROUND_OPACITY => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .background_opacity),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else {
            Err(DefaultStoragePropertiesError::UnreconizedNamespace(
                namespace.to_owned(),
            ))
        }
    }

    pub fn get_number_as_f32(
        state: &PlatformState,
        namespace: &String,
        key: &'static str,
    ) -> Result<f32, DefaultStoragePropertiesError> {
        debug!("get_number_as_f32: namespace={}, key={}", namespace, key);
        if namespace.eq(NAMESPACE_CLOSED_CAPTIONS) {
            match key {
                KEY_FONT_SIZE => Ok(state
                    .get_device_manifest()
                    .configuration
                    .default_values
                    .captions
                    .font_size),
                _ => Err(DefaultStoragePropertiesError::UnreconizedKey(
                    key.to_owned(),
                )),
            }
        } else {
            Err(DefaultStoragePropertiesError::UnreconizedNamespace(
                namespace.to_owned(),
            ))
        }
    }
}
