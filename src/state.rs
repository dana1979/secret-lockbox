use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockBox {
    pub sender: Addr,
    pub recipient: Addr,
    pub actual_amount: u128,
    pub fake_amount: u128,
    pub unlock_time: u64,
    pub is_unlocked: bool,
}

pub const LOCK_BOX: Item<LockBox> = Item::new("lockbox_state");
