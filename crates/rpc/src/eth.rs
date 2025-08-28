use std::sync::Arc;

use reth_rpc::eth::{RpcNodeCore, core::EthApiInner};
use reth_rpc_convert::RpcConvert;

/// Tempo `Eth` API implementation.
///
/// This type provides the functionality for handling `eth_` related requests.
///
/// This wraps a default `Eth` implementation, and provides additional functionality where the
/// Tempo spec deviates from the default ethereum spec, e.g. gas estimation denominated in
/// `feeToken`
///
/// This type implements the [`FullEthApi`](reth_rpc_eth_api::helpers::FullEthApi) by implemented
/// all the `Eth` helper traits and prerequisite traits.
pub struct TempoEthApi<N: RpcNodeCore, Rpc: RpcConvert> {
    /// Gateway to node's core components.
    inner: Arc<EthApiInner<N, Rpc>>,
}
