// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

pub(super) struct OpcodeBytes {
    pub(super) pushint8: Option<u8>,
    pub(super) pushint16: Option<u8>,
    pub(super) pushint32: Option<u8>,
    pub(super) pushint64: Option<u8>,
    pub(super) pushint128: Option<u8>,
    pub(super) pushm1: Option<u8>,
    pub(super) push0: Option<u8>,
    pub(super) pushdata1: Option<u8>,
    pub(super) pushdata2: Option<u8>,
    pub(super) pushdata4: Option<u8>,
    pub(super) newarray0: Option<u8>,
    pub(super) newarray: Option<u8>,
    pub(super) pack: Option<u8>,
    pub(super) drop_op: Option<u8>,
    pub(super) syscall: Option<u8>,
    pub(super) ret: Option<u8>,
}

impl OpcodeBytes {
    pub(super) fn collect() -> Self {
        use crate::opcodes;

        let get_byte = |name: &str| -> Option<u8> { opcodes::lookup(name).map(|info| info.byte) };

        Self {
            pushint8: get_byte("PUSHINT8"),
            pushint16: get_byte("PUSHINT16"),
            pushint32: get_byte("PUSHINT32"),
            pushint64: get_byte("PUSHINT64"),
            pushint128: get_byte("PUSHINT128"),
            pushm1: get_byte("PUSHM1"),
            push0: get_byte("PUSH0"),
            pushdata1: get_byte("PUSHDATA1"),
            pushdata2: get_byte("PUSHDATA2"),
            pushdata4: get_byte("PUSHDATA4"),
            newarray0: get_byte("NEWARRAY0"),
            newarray: get_byte("NEWARRAY"),
            pack: get_byte("PACK"),
            drop_op: get_byte("DROP"),
            syscall: get_byte("SYSCALL"),
            ret: get_byte("RET"),
        }
    }
}
