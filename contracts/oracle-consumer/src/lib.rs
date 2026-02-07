use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoOracleConsumer"
}"#
);

#[neo_contract]
pub struct NeoOracleConsumerContract;

#[neo_contract]
impl NeoOracleConsumerContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(owner_ptr: i64, owner_len: i64, oracle_ptr: i64, oracle_len: i64) -> bool {
        owner_ptr > 0 && owner_len > 0 && oracle_ptr > 0 && oracle_len > 0
    }

    #[neo_method]
    pub fn request(
        url_ptr: i64,
        url_len: i64,
        filter_ptr: i64,
        filter_len: i64,
        user_data_ptr: i64,
        user_data_len: i64,
    ) -> i64 {
        if url_ptr > 0
            && url_len > 0
            && filter_ptr > 0
            && filter_len > 0
            && user_data_ptr > 0
            && user_data_len > 0
        {
            1
        } else {
            0
        }
    }

    #[neo_method(name = "onOracleResponse")]
    pub fn on_oracle_response(
        request_id: i64,
        status_code: i64,
        data_ptr: i64,
        data_len: i64,
    ) -> bool {
        request_id > 0 && status_code >= 0 && data_ptr > 0 && data_len > 0
    }

    #[neo_method(name = "lastRequestId")]
    pub fn last_request_id() -> i64 {
        1
    }

    #[neo_method(name = "getConfig")]
    pub fn get_config(_unused: i64) {}

    #[neo_method(name = "getResponse")]
    pub fn get_response(_request_id: i64, _unused: i64) {}
}

impl Default for NeoOracleConsumerContract {
    fn default() -> Self {
        Self::new()
    }
}
