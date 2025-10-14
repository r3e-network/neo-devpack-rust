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

#[test]
fn overlay_macro_compiles() {
    assert!(true);
}
