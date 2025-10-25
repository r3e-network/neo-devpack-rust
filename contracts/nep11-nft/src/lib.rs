use neo_devpack::prelude::*;
use neo_devpack::utils;

const TOTAL_SUPPLY_KEY: &[u8] = b"__total_nft_supply";
const OWNER_PREFIX: &[u8] = b"owner:";
const BALANCE_PREFIX: &[u8] = b"balance:";

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP11",
    "supportedstandards": ["NEP-11"],
    "features": { "storage": true },
    "abi": {
        "methods": [
            {
                "name": "mint",
                "parameters": [
                    {"name": "owner", "type": "Integer"},
                    {"name": "token_id", "type": "Integer"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "transfer",
                "parameters": [
                    {"name": "from", "type": "Integer"},
                    {"name": "to", "type": "Integer"},
                    {"name": "token_id", "type": "Integer"}
                ],
                "returntype": "Boolean"
            }
        ],
        "events": [
            {
                "name": "Transfer",
                "parameters": [
                    {"name": "from", "type": "Integer"},
                    {"name": "to", "type": "Integer"},
                    {"name": "token_id", "type": "Integer"}
                ]
            }
        ]
    }
}"#
);

#[neo_event]
pub struct TransferEvent {
    pub from: NeoInteger,
    pub to: NeoInteger,
    pub token_id: NeoInteger,
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn owner_key(token_id: i64) -> Vec<u8> {
    let mut key = OWNER_PREFIX.to_vec();
    key.extend_from_slice(&token_id.to_le_bytes());
    key
}

fn balance_key(account: i64) -> Vec<u8> {
    let mut key = BALANCE_PREFIX.to_vec();
    key.extend_from_slice(&account.to_le_bytes());
    key
}

fn load_owner(ctx: &NeoStorageContext, token_id: i64) -> NeoResult<Option<i64>> {
    let key = NeoByteString::from_slice(&owner_key(token_id));
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        return Ok(None);
    }
    Ok(utils::bytes_to_json::<i64>(&value))
}

fn store_owner(ctx: &NeoStorageContext, token_id: i64, owner: Option<i64>) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&owner_key(token_id));
    match owner {
        Some(id) => NeoStorage::put(ctx, &key, &utils::json_to_bytes(&id)),
        None => NeoStorage::delete(ctx, &key),
    }
}

fn load_balance(ctx: &NeoStorageContext, account: i64) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(&balance_key(account));
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        Ok(0)
    } else {
        Ok(utils::bytes_to_json::<i64>(&value).unwrap_or(0))
    }
}

fn store_balance(ctx: &NeoStorageContext, account: i64, balance: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&balance_key(account));
    if balance == 0 {
        NeoStorage::delete(ctx, &key)
    } else {
        NeoStorage::put(ctx, &key, &utils::json_to_bytes(&balance))
    }
}

fn load_total_supply(ctx: &NeoStorageContext) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        Ok(0)
    } else {
        Ok(utils::bytes_to_json::<i64>(&value).unwrap_or(0))
    }
}

fn store_total_supply(ctx: &NeoStorageContext, supply: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    NeoStorage::put(ctx, &key, &utils::json_to_bytes(&supply))
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn totalSupply() -> i64 {
    storage_context()
        .and_then(|ctx| load_total_supply(&ctx).ok())
        .unwrap_or(0)
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn balanceOf(owner: i64) -> i64 {
    storage_context()
        .and_then(|ctx| load_balance(&ctx, owner).ok())
        .unwrap_or(0)
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn ownerOf(token_id: i64) -> i64 {
    storage_context()
        .and_then(|ctx| load_owner(&ctx, token_id).ok())
        .flatten()
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn mint(owner: i64, token_id: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };

    if owner <= 0 {
        return 0;
    }

    if load_owner(&ctx, token_id).ok().flatten().is_some() {
        return 0; // token already exists
    }

    let balance = load_balance(&ctx, owner).unwrap_or(0) + 1;
    if store_owner(&ctx, token_id, Some(owner)).is_err() {
        return 0;
    }
    if store_balance(&ctx, owner, balance).is_err() {
        return 0;
    }
    let supply = load_total_supply(&ctx).unwrap_or(0) + 1;
    if store_total_supply(&ctx, supply).is_err() {
        return 0;
    }

    TransferEvent {
        from: NeoInteger::new(0),
        to: NeoInteger::new(owner),
        token_id: NeoInteger::new(token_id),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn transfer(from: i64, to: i64, token_id: i64) -> i64 {
    if from == to || to <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };

    let witness = NeoByteString::from_slice(&from.to_le_bytes());
    if !NeoRuntime::check_witness(&witness)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
    {
        return 0;
    }

    let current_owner = match load_owner(&ctx, token_id).unwrap_or(None) {
        Some(owner) => owner,
        None => return 0,
    };

    if current_owner != from {
        return 0;
    }

    let from_balance = load_balance(&ctx, from).unwrap_or(0);
    let to_balance = load_balance(&ctx, to).unwrap_or(0);
    if from_balance <= 0 {
        return 0;
    }

    if store_owner(&ctx, token_id, Some(to)).is_err() {
        return 0;
    }
    if store_balance(&ctx, from, from_balance - 1).is_err() {
        return 0;
    }
    if store_balance(&ctx, to, to_balance + 1).is_err() {
        return 0;
    }

    TransferEvent {
        from: NeoInteger::new(from),
        to: NeoInteger::new(to),
        token_id: NeoInteger::new(token_id),
    }
    .emit()
    .ok();

    1
}
