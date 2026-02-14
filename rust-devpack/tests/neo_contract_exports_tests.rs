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

    #[neo_method(safe)]
    pub fn huge_value(&self) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(i64::MAX))
    }

    #[neo_method]
    pub fn fail_integer(&self) -> NeoResult<NeoInteger> {
        Err(NeoError::InvalidArgument)
    }

    #[neo_method]
    pub fn fail_boolean(&self) -> NeoResult<NeoBoolean> {
        Err(NeoError::InvalidType)
    }

    #[neo_method]
    pub fn maybe_touch(&self, should_fail: NeoBoolean) -> NeoResult<()> {
        if should_fail.as_bool() {
            Err(NeoError::InvalidOperation)
        } else {
            Ok(())
        }
    }

    #[neo_method(name = "renamedValue", safe)]
    pub fn renamed_value(&self, input: NeoInteger) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(input.as_i32_saturating() as i64 + 1))
    }

    #[neo_method(export_name = "legacyAlias")]
    pub fn legacy_alias(&self) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(9))
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
    assert_eq!(getValueLastError(), 0);

    assert_eq!(isPositive(4), 1);
    assert_eq!(isPositive(0), 0);
    assert_eq!(isPositiveLastError(), 0);

    assert_eq!(hugeValue(), i64::MAX);
    assert_eq!(hugeValueLastError(), 0);

    assert_eq!(failInteger(), 0);
    assert_eq!(
        failIntegerLastError(),
        NeoError::InvalidArgument.status_code()
    );

    assert_eq!(failBoolean(), 0);
    assert_eq!(failBooleanLastError(), NeoError::InvalidType.status_code());

    maybeTouch(0);
    assert_eq!(maybeTouchLastError(), 0);

    maybeTouch(1);
    assert_eq!(
        maybeTouchLastError(),
        NeoError::InvalidOperation.status_code()
    );

    assert_eq!(renamedValue(41), 42);
    assert_eq!(renamedValueLastError(), 0);

    assert_eq!(legacyAlias(), 9);
    assert_eq!(legacyAliasLastError(), 0);
}
