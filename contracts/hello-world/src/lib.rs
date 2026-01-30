use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "HelloWorld",
    "features": { "storage": false }
}"#
);

#[neo_safe]
#[no_mangle]
pub extern "C" fn hello() -> i64 {
    42
}

#[cfg(test)]
mod tests {
    #[test]
    fn hello_returns_42() {
        assert_eq!(super::hello(), 42);
    }
}
