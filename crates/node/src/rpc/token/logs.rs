use crate::rpc::{
    PaginationParams, TempoToken,
    logs::filter_logs,
    token::{
        role_history::{RoleChange, RoleHistoryFilters, RoleHistoryResponse},
        tokens::{Token, TokensFilters, TokensResponse},
        tokens_by_address::{AccountToken, TokensByAddressParams, TokensByAddressResponse},
    },
};
use alloy::sol_types::SolEvent;
use alloy_rpc_types_eth::Filter;
use itertools::Itertools;
use jsonrpsee::core::RpcResult;
use reth_errors::RethError;
use reth_primitives_traits::NodePrimitives;
use reth_provider::BlockIdReader;
use reth_rpc_eth_api::{EthApiTypes, RpcNodeCore, RpcNodeCoreExt, helpers::SpawnBlocking};
use reth_rpc_eth_types::EthApiError;
use reth_tracing::tracing::debug;
use reth_transaction_pool::TransactionPool;
use tempo_evm::TempoEvmConfig;
use tempo_precompiles::{
    tip20::IRolesAuth::RoleMembershipUpdated, tip20_factory::ITIP20Factory::TokenCreated,
};
use tempo_primitives::TempoHeader;

impl<EthApi> TempoToken<EthApi>
where
    EthApi: RpcNodeCore<Evm = TempoEvmConfig, Primitives: NodePrimitives<BlockHeader = TempoHeader>>
        + SpawnBlocking
        + RpcNodeCoreExt<Provider: BlockIdReader, Pool: TransactionPool>
        + EthApiTypes
        + 'static,
{
    pub async fn roles_using_logs(
        &self,
        _params: PaginationParams<RoleHistoryFilters>,
    ) -> RpcResult<RoleHistoryResponse> {
        let filter = Filter::new()
            .select(0u64..)
            .event_signature(RoleMembershipUpdated::SIGNATURE_HASH);

        let events = filter_logs(self.eth_api.clone(), filter)
            .await?
            .into_iter()
            .map(|v| v.log_decode::<RoleMembershipUpdated>());

        Ok(RoleHistoryResponse {
            next_cursor: None,
            role_changes: events
                .map_ok(|event| RoleChange {
                    timestamp: event.block_timestamp.unwrap_or_default(),
                    block_number: event.block_number.unwrap_or_default(),
                    transaction_hash: event.transaction_hash.unwrap_or_default(),
                    token: event.inner.address,
                    granted: event.inner.hasRole,
                    account: event.inner.data.account,
                    role: event.inner.data.role,
                    sender: event.inner.data.sender,
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(decode_err)?,
        })
    }

    pub async fn tokens_using_logs(
        &self,
        _params: PaginationParams<TokensFilters>,
    ) -> RpcResult<TokensResponse> {
        let filter = Filter::new()
            .select(0u64..)
            .event_signature(TokenCreated::SIGNATURE_HASH);

        let events = filter_logs(self.eth_api.clone(), filter)
            .await?
            .into_iter()
            .map(|v| v.log_decode::<TokenCreated>());

        Ok(TokensResponse {
            next_cursor: None,
            tokens: events
                .map_ok(|event| Token {
                    created_at: event.block_timestamp.unwrap_or_default(),
                    address: event.inner.token,
                    creator: event.inner.address,
                    decimals: 2,
                    paused: false,
                    quote_token: event.inner.token,
                    supply_cap: 0,
                    token_id: event.inner.tokenId,
                    currency: event.inner.currency.clone(),
                    name: event.inner.name.clone(),
                    symbol: event.inner.symbol.clone(),
                    total_supply: 0,
                    transfer_policy_id: 0,
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(decode_err)?,
        })
    }

    pub async fn tokens_by_addresses_using_logs(
        &self,
        params: TokensByAddressParams,
    ) -> RpcResult<TokensByAddressResponse> {
        let filter = Filter::new()
            .select(0u64..)
            .event_signature(TokenCreated::SIGNATURE_HASH)
            .topic1(params.address);

        let events = filter_logs(self.eth_api.clone(), filter)
            .await?
            .into_iter()
            .map(|v| v.log_decode::<TokenCreated>());

        Ok(TokensByAddressResponse {
            next_cursor: None,
            tokens: events
                .map_ok(|event| AccountToken {
                    balance: Default::default(),
                    roles: vec![],
                    token: Token {
                        created_at: event.block_timestamp.unwrap_or_default(),
                        address: event.inner.token,
                        creator: event.inner.address,
                        decimals: 2,
                        paused: false,
                        quote_token: event.inner.token,
                        supply_cap: 0,
                        token_id: event.inner.tokenId,
                        currency: event.inner.currency.clone(),
                        name: event.inner.name.clone(),
                        symbol: event.inner.symbol.clone(),
                        total_supply: 0,
                        transfer_policy_id: 0,
                    },
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(decode_err)?,
        })
    }
}

fn decode_err(err: alloy::sol_types::Error) -> EthApiError {
    debug!(
        target: "rpc::token",
        ?err,
        "decode logs"
    );

    EthApiError::Internal(RethError::Other(Box::new(err)))
}
