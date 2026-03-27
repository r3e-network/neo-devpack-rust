// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "HelloWorld"
}"#
);

const HELLO_RESULT: i64 = 42;

#[neo_contract]
pub struct HelloWorldContract;

#[neo_contract]
impl HelloWorldContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method(safe)]
    pub fn hello() -> i64 {
        HELLO_RESULT
    }
}

impl Default for HelloWorldContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hello_returns_42() {
        assert_eq!(super::HelloWorldContract::hello(), 42);
    }
}
