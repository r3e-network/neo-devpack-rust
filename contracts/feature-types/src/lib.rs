// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the entire `neo-types` value-type surface —
//! `NeoByteString`, `NeoString`, `NeoBoolean`, `NeoArray`, `NeoMap`,
//! `NeoStruct`, `NeoValue` (+ all `From` impls and accessors), `Hash160`,
//! `Hash256`, `NeoIterator`, the `serialise_*` helpers, and the size
//! constants. Every one of these is implemented in pure Rust and compiles to
//! plain wasm (no syscalls), so each method's result is computed inside the
//! body and reduced to a scalar `i64` / `bool` for the export boundary.

extern crate alloc;
use alloc::vec;
use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureTypes" }"#);

#[neo_contract]
pub struct TypesContract;

#[neo_contract]
impl TypesContract {
    pub fn new() -> Self {
        Self
    }

    /// `NeoByteString`: new/from_slice/as_slice/len/is_empty/push/
    /// extend_from_slice/try_push/try_extend_from_slice + MAX_SIZE/max_size.
    #[neo_method(safe)]
    pub fn bs_ops(seed: i64) -> i64 {
        let mut a = NeoByteString::from_slice(&seed.to_le_bytes());
        let mut acc = a.len() as i64 + a.is_empty() as i64;
        a.push((seed & 0xFF) as u8);
        a.extend_from_slice(b"xy");
        acc += a.as_slice().iter().map(|&b| b as i64).sum::<i64>();
        let mut b = NeoByteString::new(vec![1u8, 2, 3]);
        let _ = b.try_push(4);
        let _ = b.try_extend_from_slice(&[5, 6]);
        acc += b.len() as i64;
        acc += NeoByteString::MAX_SIZE as i64 + NeoByteString::max_size() as i64;
        acc
    }

    /// `NeoString`: from_str/new/as_str/len/is_empty + `PartialEq<&str>`.
    #[neo_method(safe)]
    pub fn str_ops() -> i64 {
        let s = NeoString::from_str("neo");
        let t = NeoString::new(alloc::string::String::from("vm"));
        let mut acc = s.len() as i64 + t.len() as i64;
        acc += s.is_empty() as i64;
        acc += (s == "neo") as i64;
        acc += s.as_str().bytes().map(|b| b as i64).sum::<i64>();
        acc
    }

    /// `NeoBoolean`: TRUE/FALSE/new/as_bool + BitAnd/BitOr/BitXor/Not +
    /// From<bool>/From<NeoBoolean>.
    #[neo_method(safe)]
    pub fn bool_ops(c: bool) -> bool {
        let t = NeoBoolean::TRUE;
        let f = NeoBoolean::FALSE;
        let x = NeoBoolean::new(c);
        let and = (t & x).as_bool();
        let or = (f | x).as_bool();
        let xor = (t ^ x).as_bool();
        let not = (!x).as_bool();
        let from: NeoBoolean = c.into();
        let back: bool = from.into();
        and == or && (xor != not) || back
    }

    /// `NeoArray`: new/with_capacity/from_vec/push/pop/len/is_empty/get/
    /// get_mut/iter/try_push/remaining_capacity + MAX_SIZE.
    #[neo_method(safe)]
    pub fn arr_ops() -> i64 {
        let mut a: NeoArray<i64> = NeoArray::new();
        a.push(10);
        a.push(20);
        let mut b: NeoArray<i64> = NeoArray::with_capacity(4);
        let _ = b.try_push(1);
        let c = NeoArray::from_vec(vec![3i64, 4, 5]);
        if let Some(x) = a.get_mut(0) {
            *x += 5;
        }
        let popped = a.pop().unwrap_or(0);
        let mut acc = a.len() as i64 + b.len() as i64 + c.len() as i64;
        acc += a.is_empty() as i64;
        acc += a.get(0).copied().unwrap_or(0) + popped;
        acc += c.iter().copied().sum::<i64>();
        acc += b.remaining_capacity() as i64;
        acc += (NeoArray::<i64>::MAX_SIZE > 0) as i64;
        acc
    }

    /// `NeoMap`: new/insert/get/get_mut/remove/len/is_empty/iter/
    /// contains_key/keys/values/remove_strict.
    #[neo_method(safe)]
    pub fn map_ops() -> i64 {
        let mut m: NeoMap<NeoInteger, NeoInteger> = NeoMap::new();
        m.insert(NeoInteger::new(1), NeoInteger::new(100));
        m.insert(NeoInteger::new(2), NeoInteger::new(200));
        if let Some(v) = m.get_mut(&NeoInteger::new(1)) {
            *v = NeoInteger::new(150);
        }
        let mut acc = m.len() as i64 + m.is_empty() as i64;
        acc += m
            .get(&NeoInteger::new(1))
            .map(|v| v.as_i64_saturating())
            .unwrap_or(0);
        acc += m.contains_key(&NeoInteger::new(2)) as i64;
        acc += m.keys().count() as i64 + m.values().count() as i64;
        acc += m.iter().count() as i64;
        let _ = m.remove_strict(&NeoInteger::new(2), &NeoInteger::new(200));
        acc += m.remove(&NeoInteger::new(1)).is_some() as i64;
        acc += m.len() as i64;
        acc
    }

    /// `NeoStruct`: new/with_field/get_field/set_field/insert/len/is_empty/iter.
    #[neo_method(safe)]
    pub fn struct_ops() -> i64 {
        let mut s = NeoStruct::new()
            .with_field("a", NeoValue::from(1i64))
            .with_field("b", NeoValue::from(2i64));
        s.set_field("a", NeoValue::from(10i64));
        s.insert(NeoString::from_str("c"), NeoValue::from(3i64));
        let mut acc = s.len() as i64 + s.is_empty() as i64;
        acc += s
            .get_field("a")
            .and_then(|v| v.as_integer())
            .map(|n| n.as_i64_saturating())
            .unwrap_or(0);
        acc += s.iter().count() as i64;
        acc
    }

    /// `NeoValue`: every `From` variant + is_null/as_integer/as_boolean/
    /// as_byte_string/as_string/as_array/as_map/as_struct.
    #[neo_method(safe)]
    pub fn value_ops() -> i64 {
        let mut acc = 0i64;
        let vi = NeoValue::from(NeoInteger::new(7));
        let vi64 = NeoValue::from(42i64);
        let vb = NeoValue::from(NeoBoolean::TRUE);
        let vbool = NeoValue::from(true);
        let vbs = NeoValue::from(NeoByteString::from_slice(b"ab"));
        let vs = NeoValue::from(NeoString::from_str("s"));
        let varr = NeoValue::from(NeoArray::<NeoValue>::new());
        let vmap = NeoValue::from(NeoMap::<NeoValue, NeoValue>::new());
        let vst = NeoValue::from(NeoStruct::new());
        let vnull = NeoValue::Null;

        acc += vi.as_integer().map(|n| n.as_i64_saturating()).unwrap_or(0);
        acc += vi64.as_integer().map(|n| n.as_i64_saturating()).unwrap_or(0);
        acc += vb.as_boolean().map(|b| b.as_bool() as i64).unwrap_or(0);
        acc += vbool.as_boolean().map(|b| b.as_bool() as i64).unwrap_or(0);
        acc += vbs.as_byte_string().map(|b| b.len() as i64).unwrap_or(0);
        acc += vs.as_string().map(|s| s.len() as i64).unwrap_or(0);
        acc += varr.as_array().map(|a| a.len() as i64).unwrap_or(0);
        acc += vmap.as_map().map(|m| m.len() as i64).unwrap_or(0);
        acc += vst.as_struct().map(|s| s.len() as i64).unwrap_or(0);
        acc += vnull.is_null() as i64;
        acc
    }

    /// `Hash160`: ZERO/LENGTH/from_bytes/try_from_slice/as_bytes/as_slice/
    /// is_zero/to_byte_string + TryFrom<&[u8]>/From<Hash160> for NeoByteString.
    #[neo_method(safe)]
    pub fn hash160_ops() -> i64 {
        let z = Hash160::ZERO;
        let h = Hash160::from_bytes([1u8; 20]);
        let mut acc = Hash160::LENGTH as i64 + z.is_zero() as i64;
        acc += h.as_bytes().iter().map(|&b| b as i64).sum::<i64>();
        acc += h.as_slice().len() as i64;
        acc += h.to_byte_string().len() as i64;
        if let Ok(h2) = Hash160::try_from_slice(&[2u8; 20]) {
            acc += h2.as_slice()[0] as i64;
        }
        let bs: NeoByteString = h.into();
        acc += bs.len() as i64;
        if let Ok(h3) = Hash160::try_from(&[3u8; 20][..]) {
            acc += h3.as_slice()[0] as i64;
        }
        acc
    }

    /// `Hash256`: same surface as Hash160 (32-byte).
    #[neo_method(safe)]
    pub fn hash256_ops() -> i64 {
        let z = Hash256::ZERO;
        let h = Hash256::from_bytes([1u8; 32]);
        let mut acc = Hash256::LENGTH as i64 + z.is_zero() as i64;
        acc += h.as_bytes().iter().map(|&b| b as i64).sum::<i64>();
        acc += h.as_slice().len() as i64 + h.to_byte_string().len() as i64;
        if let Ok(h2) = Hash256::try_from_slice(&[2u8; 32]) {
            acc += h2.as_slice()[0] as i64;
        }
        let bs: NeoByteString = h.into();
        acc += bs.len() as i64;
        acc
    }

    /// `NeoInteger`: zero/one/min_i32/max_i32/MAX_BYTE_LENGTH/as_bigint/
    /// to_bigint/from_bigint + BigInt re-export.
    #[neo_method(safe)]
    pub fn int_consts() -> i64 {
        let mut acc = NeoInteger::zero().as_i64_saturating()
            + NeoInteger::one().as_i64_saturating()
            + NeoInteger::MAX_BYTE_LENGTH as i64;
        acc += NeoInteger::min_i32().as_i64_saturating() + NeoInteger::max_i32().as_i64_saturating();
        let n = NeoInteger::new(123);
        let big: BigInt = n.to_bigint();
        let from = NeoInteger::from_bigint(&big);
        acc += from.as_i64_saturating();
        acc += (n.as_bigint() == &big) as i64;
        acc
    }

    /// `serialise_value` / `serialise_array` / `serialise_notification` +
    /// MAX_NOTIFICATION_SIZE / MAX_STACK_SIZE.
    #[neo_method(safe)]
    pub fn serialise_ops() -> i64 {
        let v = NeoValue::from(99i64);
        let mut arr: NeoArray<NeoValue> = NeoArray::new();
        arr.push(NeoValue::from(1i64));
        arr.push(NeoValue::from(NeoByteString::from_slice(b"z")));
        let name = NeoString::from_str("Transfer");
        let bytes_v = serialise_value(&v).len() as i64;
        let bytes_a = serialise_array(&arr).len() as i64;
        let bytes_n = serialise_notification(&name, &arr).len() as i64;
        bytes_v
            + bytes_a
            + bytes_n
            + (MAX_NOTIFICATION_SIZE > 0) as i64
            + (MAX_STACK_SIZE > 0) as i64
    }

    /// `NeoIterator`: new/has_next/len/is_empty/reset/total_len + the
    /// `Iterator` impl (sum via next()).
    #[neo_method(safe)]
    pub fn iter_ops() -> i64 {
        let mut it = NeoIterator::new(vec![1i64, 2, 3, 4]);
        let mut acc = it.total_len() as i64 + it.len() as i64 + it.is_empty() as i64;
        acc += it.has_next() as i64;
        let first = it.next().unwrap_or(0);
        acc += first;
        it.reset();
        let sum: i64 = it.sum();
        acc + sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_surface_is_deterministic() {
        // Same input -> same output (pure functions).
        assert_eq!(TypesContract::bs_ops(5), TypesContract::bs_ops(5));
        assert!(TypesContract::map_ops() > 0);
        assert!(TypesContract::value_ops() > 0);
        assert_eq!(TypesContract::hash160_ops(), TypesContract::hash160_ops());
        assert!(TypesContract::iter_ops() > 0);
        assert!(TypesContract::serialise_ops() > 0);
    }
}
