// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{"permissions": []}"#);

#[neo_event]
pub struct SampleEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

neo_permission!("0xff", ["balanceOf"]);
neo_supported_standards!(["NEP-17"]);
neo_trusts!(["*"]);
neo_manifest_overlay!(
    r#"{
    "extra": {
        "author": "Neo Team",
        "description": "syntax coverage",
        "version": "1.2.3",
        "customField": {"channel": "ci"}
    },
    "permissions": [
        {
            "contract": "0xff",
            "methods": "*",
            "labels": ["runtime", "wildcard"]
        }
    ]
}"#
);

#[test]
fn overlay_macro_compiles() {}
