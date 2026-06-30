// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//! End-to-end: compile real sample contracts and run their methods on a real
//! NeoVM, asserting the actual on-chain return values.
use neo_vm_test::Contract;

#[test]
fn arithmetic_on_real_vm() {
    let c = Contract::compile("contracts/feature-arithmetic").expect("compile");
    // (a+b)*3 - b/2 + a%7
    c.invoke("calc", &[20.into(), 6.into()]).assert_returns_i64(81);
    // clz(56)+ctz(0)+popcnt(8) for 255 == 64  (the popcount-wraparound fix)
    c.invoke("bitcount", &[255.into()]).assert_returns_i64(64);
    // !((a&b)|(a^b))
    c.invoke("bits", &[12.into(), 10.into()]).assert_returns_i64(-15);
    // i32 wrap of 0x1_0000_0007
    c.invoke("narrow", &[4_294_967_303i64.into()]).assert_returns_i64(7);
    // NeoInteger chain (the eqz-class fix unblocked BigInt math)
    c.invoke("bigmath", &[100.into(), 7.into()]).assert_returns_i64(40);
}

#[test]
fn control_flow_on_real_vm() {
    let c = Contract::compile("contracts/feature-control-flow").expect("compile");
    c.invoke("sel", &[3.into()]).assert_returns_i64(103); // br_table
    c.invoke("fib", &[10.into()]).assert_returns_i64(55); // recursion
    c.invoke("indirect", &[1.into(), 5.into()]).assert_returns_i64(7); // call_indirect
    c.invoke("pick", &[true.into(), 10.into(), 20.into()]).assert_returns_i64(11); // select
}
