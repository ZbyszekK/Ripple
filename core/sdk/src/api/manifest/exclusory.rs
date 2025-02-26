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

use std::collections::HashMap;

use serde::Deserialize;

use super::device_manifest::DeviceManifest;

#[derive(Debug, Clone, Deserialize)]
pub struct AppAuthorizationRules {
    pub app_ignore_rules: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExclusoryImpl {
    pub resolve_only: Option<Vec<String>>,
    pub app_authorization_rules: AppAuthorizationRules,
    /*
    method names to ignore regardless of appid
    */
    pub method_ignore_rules: Vec<String>,
}

impl ExclusoryImpl {
    pub fn get(dm: DeviceManifest) -> ExclusoryImpl {
        if let Some(e) = dm.configuration.exclusory {
            return e;
        }
        ExclusoryImpl {
            resolve_only: None,
            method_ignore_rules: vec!["*".into()],
            app_authorization_rules: AppAuthorizationRules {
                app_ignore_rules: HashMap::new(),
            },
        }
    }
}

impl Exclusory for ExclusoryImpl {
    fn can_resolve(&self, method: String) -> bool {
        if let Some(r) = &self.resolve_only {
            if !r.is_empty() {
                return r.contains(&method);
            }
        }
        true
    }

    fn is_excluded(&self, app_id: String, method: String) -> bool {
        /*dangerous, but possible... * method matcher,let em all in  */
        if self.clone().is_all_excluded() || self.clone().is_method_excluded(method.clone()) {
            return true;
        };

        let default: Vec<String> = vec![];
        let methods = self
            .app_authorization_rules
            .app_ignore_rules
            .get(&app_id)
            .unwrap_or(&default);
        if methods.contains(&String::from("*")) {
            return true;
        };

        methods.contains(&method)
    }

    fn is_app_all_excluded(&self, app_id: &str) -> bool {
        if let Some(app) = self.app_authorization_rules.app_ignore_rules.get(app_id) {
            return app.contains(&String::from("*"));
        }
        false
    }

    fn is_all_excluded(&self) -> bool {
        self.method_ignore_rules.contains(&String::from("*"))
    }

    fn is_method_excluded(&self, method: String) -> bool {
        self.method_ignore_rules.contains(&method)
    }
}
pub trait Exclusory {
    fn is_excluded(&self, app_id: String, method: String) -> bool;
    fn is_all_excluded(&self) -> bool;
    fn is_method_excluded(&self, method: String) -> bool;
    fn can_resolve(&self, method: String) -> bool;
    fn is_app_all_excluded(&self, app_id: &str) -> bool;
}
