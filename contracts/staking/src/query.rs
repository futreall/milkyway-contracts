use crate::helpers::paginate_map;
use crate::msg::{
    BatchResponse, BatchesResponse, ConfigResponse, IBCQueueResponse, IBCReplyQueueResponse,
    LiquidUnstakeRequestResponse, StateResponse,
};
use crate::state::ibc::IBCTransfer;
use crate::state::{
    BATCHES, CONFIG, IBC_WAITING_FOR_REPLY, INFLIGHT_PACKETS, PENDING_BATCH_ID, STATE,
};
use cosmwasm_std::{Addr, Decimal, Deps, StdResult, Timestamp, Uint128};
use milky_way::staking::Batch;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        native_token_denom: config.native_token_denom,
        liquid_stake_token_denom: config.liquid_stake_token_denom,
        treasury_address: config.treasury_address.to_string(),
        operators: config
            .operators
            .into_iter()
            .map(|v| v.to_string())
            .collect(),
        validators: config
            .validators
            .into_iter()
            .map(|v| v.to_string())
            .collect(),
        batch_period: config.batch_period,
        unbonding_period: config.unbonding_period,
        minimum_liquid_stake_amount: config.minimum_liquid_stake_amount,
        staker_address: config.multisig_address_config.staker_address.to_string(),
        reward_collector_address: config
            .multisig_address_config
            .reward_collector_address
            .to_string(),
    };
    Ok(res)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    let res = StateResponse {
        total_native_token: state.total_native_token,
        total_liquid_stake_token: state.total_liquid_stake_token,
        rate: if state.total_native_token == Uint128::zero() {
            Decimal::zero()
        } else {
            Decimal::from_ratio(state.total_liquid_stake_token, state.total_native_token)
        },
        pending_owner: state
            .pending_owner
            .map(|v| v.to_string())
            .unwrap_or_default(),
        total_reward_amount: state.total_reward_amount,
    };
    Ok(res)
}

fn batch_to_response(batch: Batch) -> BatchResponse {
    BatchResponse {
        id: batch.id,
        batch_total_liquid_stake: batch.batch_total_liquid_stake,
        expected_native_unstaked: batch.expected_native_unstaked.unwrap_or(Uint128::zero()),
        received_native_unstaked: batch.received_native_unstaked.unwrap_or(Uint128::zero()),
        next_batch_action_time: Timestamp::from_seconds(
            batch.next_batch_action_time.unwrap_or(0u64),
        ),
        status: batch.status.as_str().to_string(),
        requests: batch
            .liquid_unstake_requests
            .into_iter()
            .map(|v| LiquidUnstakeRequestResponse {
                user: v.1.user.to_string(),
                amount: v.1.shares,
                redeemed: v.1.redeemed,
            })
            .collect(),
    }
}

pub fn query_batch(deps: Deps, id: u64) -> StdResult<BatchResponse> {
    let batch: Batch = BATCHES.load(deps.storage, id)?;
    Ok(batch_to_response(batch))
}

pub fn query_batches(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<BatchesResponse> {
    let batches = paginate_map(
        deps,
        &BATCHES,
        start_after,
        limit,
        cosmwasm_std::Order::Ascending,
        None,
    )?;

    let res = BatchesResponse {
        batches: batches.into_iter().map(|v| batch_to_response(v)).collect(),
    };
    Ok(res)
}

pub fn query_pending_batch(deps: Deps) -> StdResult<BatchResponse> {
    let pending_batch_id = PENDING_BATCH_ID.load(deps.storage)?;
    let pending_batch = BATCHES.load(deps.storage, pending_batch_id)?;

    Ok(batch_to_response(pending_batch))
}

pub fn query_ibc_queue(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<IBCQueueResponse> {
    let inflight_packets: Vec<IBCTransfer> = paginate_map(
        deps,
        &INFLIGHT_PACKETS,
        start_after,
        limit,
        cosmwasm_std::Order::Ascending,
        None,
    )?;
    let res = IBCQueueResponse {
        ibc_queue: inflight_packets,
    };

    Ok(res)
}

pub fn query_reply_queue(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<IBCReplyQueueResponse> {
    let ibc_messages_waiting = paginate_map(
        deps,
        &IBC_WAITING_FOR_REPLY,
        start_after,
        limit,
        cosmwasm_std::Order::Ascending,
        None,
    )?;
    let res = IBCReplyQueueResponse {
        ibc_queue: ibc_messages_waiting,
    };
    Ok(res)
}

pub fn query_claimable(
    deps: Deps,
    user: Addr,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<BatchesResponse> {
    deps.api.addr_validate(&user.to_string())?;

    let batches = paginate_map(
        deps,
        &BATCHES,
        start_after,
        limit,
        cosmwasm_std::Order::Ascending,
        Some(|b| return b.status == milky_way::staking::BatchStatus::Received),
    )?
    .into_iter()
    .filter(|b| {
        !b.liquid_unstake_requests
            .get(&user.to_string())
            .unwrap()
            .redeemed
    })
    .map(|v| batch_to_response(v))
    .collect();

    let res = BatchesResponse { batches };
    Ok(res)
}
