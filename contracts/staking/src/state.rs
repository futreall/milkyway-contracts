use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use milky_way::staking::Batch;

#[cw_serde]
pub struct Config {
    pub native_token_denom: String,
    pub liquid_stake_token_denom: String,
    pub treasury_address: Addr,
    pub node_operators: Vec<Addr>,
    pub validators: Vec<Addr>,
    pub batch_period: u64,
    pub unbonding_period: u64,
    pub protocol_fee_config: ProtocolFeeConfig,
    pub multisig_address_config: MultisigAddressConfig,
    pub minimum_liquid_stake_amount: Uint128,
    pub minimum_rewards_to_collect: Uint128,
}
// TODO: PENDING - DOCS DEFINE THESE AS MAPS?
// Discuss: Do we want to add or remove any state?
#[cw_serde]
pub struct State {
    pub total_native_token: Uint128,
    pub total_liquid_stake_token: Uint128,
    pub native_token_to_stake: Uint128,
    pub pending_owner: Option<Addr>,
}

#[cw_serde]
pub struct ProtocolFeeConfig {
    pub dao_treasury_fee: Uint128,
}

#[cw_serde]
pub struct MultisigAddressConfig {
    pub controller_address: Addr,
    pub staker_address: Addr,
    pub reward_collector_address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("state");
// TODO: Finalize and discuss batch structure
pub const BATCHES: Map<u64, Batch> = Map::new("batches");
// Only one batch can be pending at a time in the current design
pub const PENDING_BATCH: Item<Batch> = Item::new("pending_batch");