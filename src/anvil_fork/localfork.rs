use std::time::Duration;

// File copied from:
// https://github.com/foundry-rs/foundry/blob/master/anvil/tests/it/fork.rs
use anvil::{eth::EthApi, spawn, NodeConfig, NodeHandle};

use crate::SETTINGS;


const BLOCK_NUMBER: u64 = 17_296_515u64;

/// Represents an anvil fork of an anvil node
#[allow(unused)]
pub struct LocalFork {
    origin_api: EthApi,
    origin_handle: NodeHandle,
    fork_api: EthApi,
    fork_handle: NodeHandle,
}

// === impl LocalFork ===
#[allow(dead_code)]
impl LocalFork {
    /// Spawns two nodes with the test config
    pub async fn new() -> Self {
        Self::setup(NodeConfig::test(), NodeConfig::test()).await
    }

    /// Spawns two nodes where one is a fork of the other
    pub async fn setup(origin: NodeConfig, fork: NodeConfig) -> Self {
        let (origin_api, origin_handle) = spawn(origin).await;

        let (fork_api, fork_handle) =
            spawn(fork.with_eth_rpc_url(Some(origin_handle.http_endpoint()))).await;
        Self { origin_api, origin_handle, fork_api, fork_handle }
    }
}

pub fn fork_config() -> NodeConfig {
    NodeConfig::default()
        .with_eth_rpc_url(Some(&SETTINGS.rpc_url))
        .with_fork_block_number(Some(BLOCK_NUMBER))
        .with_blocktime(Some(Duration::from_secs(1)))
}
