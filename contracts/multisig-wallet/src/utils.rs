use core::slice;
use neo_devpack::prelude::*;

pub fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(slice.to_vec())
}

pub fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

pub fn read_string(ptr: i64, len: i64) -> Option<String> {
    let bytes = read_bytes(ptr, len)?;
    String::from_utf8(bytes).ok()
}

pub fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

pub fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}
