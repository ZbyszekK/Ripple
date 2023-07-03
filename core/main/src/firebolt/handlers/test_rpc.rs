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
    firebolt::rpc::RippleRPCProvider,
    state::platform_state::PlatformState,
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    tracing::info,
    RpcModule,
};

use ripple_sdk::api::{
    gateway::rpc_gateway_api::CallContext,
};

#[rpc(server)]
pub trait TestAPI {
    #[method(name = "test.test")]
    async fn test_api(&self, ctx: CallContext) -> RpcResult<String>;
}

pub struct TestAPIImpl {
    pub state: PlatformState,
}

#[async_trait]
impl TestAPIServer for TestAPIImpl {
    async fn test_api(
        &self,
        _ctx: CallContext
    ) -> RpcResult<String> {
        info!("Inside TestAPIServer::test_api");
        return Ok(String::from("echo"));
    }

}

pub struct TestRPCProvider;
impl RippleRPCProvider<TestAPIImpl> for TestRPCProvider {
    fn provide(state: PlatformState) -> RpcModule<TestAPIImpl> {
        (TestAPIImpl { state }).into_rpc()
    }
}
