use neo_devpack::prelude::*;

use crate::storage::*;
use crate::types::DaoConfig;

pub fn load_config(ctx: &NeoStorageContext) -> Option<DaoConfig> {
    load_from_storage(ctx, CONFIG_KEY)
}

pub fn store_config(ctx: &NeoStorageContext, config: &DaoConfig) -> NeoResult<()> {
    store_to_storage(ctx, CONFIG_KEY, config)
}

pub fn load_stake(ctx: &NeoStorageContext, address: &NeoByteString) -> i64 {
    load_from_storage(ctx, &stake_key(address)).unwrap_or(0i64)
}

pub fn store_stake(ctx: &NeoStorageContext, address: &NeoByteString, amount: i64) -> NeoResult<()> {
    if amount == 0 {
        let key = NeoByteString::from_slice(&stake_key(address));
        NeoStorage::delete(ctx, &key)
    } else {
        store_to_storage(ctx, &stake_key(address), &amount)
    }
}
