// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone)]
pub(super) enum Literal {
    Integer(i128),
    Bytes(Vec<u8>),
    Array(usize),
    Unknown,
}
