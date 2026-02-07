// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_devpack::prelude::*;

#[neo_contract]
pub struct ExportedContract;

#[neo_contract]
impl ExportedContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe)]
    pub fn get_value(&self, input: NeoInteger) -> NeoResult<NeoInteger> {
        Ok(input)
    }

    #[neo_method]
    pub fn is_positive(&self, input: NeoInteger) -> NeoResult<NeoBoolean> {
        Ok(NeoBoolean::new(input.as_i32_saturating() as i64 > 0))
    }
}

impl Default for ExportedContract {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn generated_exports_are_callable() {
    assert_eq!(getValue(7), 7);
    assert_eq!(isPositive(4), 1);
    assert_eq!(isPositive(0), 0);
}
