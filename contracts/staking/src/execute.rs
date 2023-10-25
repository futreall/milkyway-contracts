use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::helpers::compute_mint_amount;
use crate::state::{ADMIN, CONFIG, STATE};
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgMint;

// Payment validation handled by caller
// Denom validation handled by caller
pub fn execute_liquid_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    ensure!(
        amount > config.minimum_liquid_stake_amount,
        ContractError::MinimumLiquidStakeAmount {
            minimum_stake_amount: (config.minimum_liquid_stake_amount),
            sent_amount: (amount)
        }
    );

    //Compute mint amount
    let mint_amount = compute_mint_amount(
        state.total_native_token,
        state.total_liquid_stake_token,
        amount,
    );

    if mint_amount.is_zero() {
        return Err(ContractError::MintError {});
    }
    // TODO: Confirm Uint128 to String conversion is ok (proto requires this)
    // TODO: Needs testing and validation - also need to check mint_to_address
    // Mint liquid staking token
    let mint_msg = MsgMint {
        sender: env.contract.address.to_string(),
        amount: Some(Coin {
            denom: config.liquid_stake_token_denom,
            amount: mint_amount.to_string(),
        }),
        mint_to_address: info.sender.to_string(),
    };

    // TODO: Add IBC logic
    //Transfer native token to multisig address
    // <<INSERT IBC LOGIC HERE>>

    state.total_native_token += amount;
    state.total_liquid_stake_token += mint_amount;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "liquid_stake")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", amount))
}

pub fn execute_liquid_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    unimplemented!()
}

pub fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    unimplemented!()
}
// Transfer ownership to another account; callable by the owner
// This will require the new owner to accept to take effect.
// No need to handle case of overwriting the pending owner
pub fn execute_transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> ContractResult<Response> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    state.pending_owner = Some(deps.api.addr_validate(&new_owner)?);

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner)
        .add_attribute("previous_owner", info.sender))
}
// Revoke transfer ownership, callable by the owner
pub fn execute_revoke_ownership_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    state.pending_owner = None;

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "revoke_ownership_transfer"))
}

pub fn execute_accept_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let new_owner = {
        let mut state = STATE.load(deps.storage)?;
        match state.pending_owner {
            Some(pending_owner) if pending_owner == info.sender => {
                state.pending_owner = None;
                STATE.save(deps.storage, &state)?;
                Some(pending_owner)
            }
            _ => None,
        }
    };

    match new_owner {
        Some(pending_owner) => {
            ADMIN.set(deps, Some(pending_owner))?;
            Ok(Response::new()
                .add_attribute("action", "accept_ownership")
                .add_attribute("new_owner", info.sender))
        }
        None => Err(ContractError::NoPendingOwner {}),
    }
}

pub fn execute_add_validator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_validator: String,
) -> ContractResult<Response> {
    unimplemented!()
}

pub fn execute_remove_validator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: String,
) -> ContractResult<Response> {
    unimplemented!()
}
