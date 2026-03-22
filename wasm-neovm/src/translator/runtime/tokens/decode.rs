// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::literal::Literal;
use super::opcodes::OpcodeBytes;

pub(super) enum TokenScanEvent {
    Continue,
    Syscall(u32),
    Stop,
}

pub(super) fn step(
    op: u8,
    script: &[u8],
    pc: &mut usize,
    ops: &OpcodeBytes,
    stack: &mut Vec<Literal>,
) -> TokenScanEvent {
    let literal = if Some(op) == ops.pushm1 {
        Some(Literal::Integer(-1))
    } else if let Some(p0) = ops.push0 {
        if op >= p0 && op <= p0 + 16 {
            Some(Literal::Integer((op - p0) as i128))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(lit) = literal {
        stack.push(lit);
        return TokenScanEvent::Continue;
    }

    if Some(op) == ops.pushint8 {
        if *pc + 1 > script.len() {
            return TokenScanEvent::Stop;
        }
        let value = i8::from_le_bytes([script[*pc]]);
        *pc += 1;
        stack.push(Literal::Integer(value.into()));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushint16 {
        if *pc + 2 > script.len() {
            return TokenScanEvent::Stop;
        }
        let value = i16::from_le_bytes([script[*pc], script[*pc + 1]]);
        *pc += 2;
        stack.push(Literal::Integer(value.into()));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushint32 {
        if *pc + 4 > script.len() {
            return TokenScanEvent::Stop;
        }
        let value = i32::from_le_bytes([
            script[*pc],
            script[*pc + 1],
            script[*pc + 2],
            script[*pc + 3],
        ]);
        *pc += 4;
        stack.push(Literal::Integer(value.into()));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushint64 {
        if *pc + 8 > script.len() {
            return TokenScanEvent::Stop;
        }
        let value = i64::from_le_bytes([
            script[*pc],
            script[*pc + 1],
            script[*pc + 2],
            script[*pc + 3],
            script[*pc + 4],
            script[*pc + 5],
            script[*pc + 6],
            script[*pc + 7],
        ]);
        *pc += 8;
        stack.push(Literal::Integer(value.into()));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushint128 {
        if *pc + 16 > script.len() {
            return TokenScanEvent::Stop;
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&script[*pc..*pc + 16]);
        *pc += 16;
        let value = i128::from_le_bytes(bytes);
        stack.push(Literal::Integer(value));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushdata1 {
        if *pc >= script.len() {
            return TokenScanEvent::Stop;
        }
        let len = script[*pc] as usize;
        *pc += 1;
        if *pc + len > script.len() {
            return TokenScanEvent::Stop;
        }
        let data = script[*pc..*pc + len].to_vec();
        *pc += len;
        stack.push(Literal::Bytes(data));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushdata2 {
        if *pc + 2 > script.len() {
            return TokenScanEvent::Stop;
        }
        let len = u16::from_le_bytes([script[*pc], script[*pc + 1]]) as usize;
        *pc += 2;
        if *pc + len > script.len() {
            return TokenScanEvent::Stop;
        }
        let data = script[*pc..*pc + len].to_vec();
        *pc += len;
        stack.push(Literal::Bytes(data));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pushdata4 {
        if *pc + 4 > script.len() {
            return TokenScanEvent::Stop;
        }
        let len = u32::from_le_bytes([
            script[*pc],
            script[*pc + 1],
            script[*pc + 2],
            script[*pc + 3],
        ]) as usize;
        *pc += 4;
        if *pc + len > script.len() {
            return TokenScanEvent::Stop;
        }
        let data = script[*pc..*pc + len].to_vec();
        *pc += len;
        stack.push(Literal::Bytes(data));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.newarray0 {
        stack.push(Literal::Array(0));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.newarray {
        let count = match stack.pop() {
            Some(Literal::Integer(v)) => v,
            _ => {
                stack.push(Literal::Unknown);
                return TokenScanEvent::Continue;
            }
        };
        let count = match usize::try_from(count) {
            Ok(v) => v,
            Err(_) => {
                stack.push(Literal::Unknown);
                return TokenScanEvent::Continue;
            }
        };
        for _ in 0..count {
            if stack.pop().is_none() {
                stack.clear();
                return TokenScanEvent::Continue;
            }
        }
        stack.push(Literal::Array(count));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.pack {
        let count = match stack.pop() {
            Some(Literal::Integer(v)) => v,
            _ => {
                stack.push(Literal::Unknown);
                return TokenScanEvent::Continue;
            }
        };
        let count = match usize::try_from(count) {
            Ok(v) => v,
            Err(_) => {
                stack.push(Literal::Unknown);
                return TokenScanEvent::Continue;
            }
        };
        if stack.len() < count {
            stack.clear();
            return TokenScanEvent::Continue;
        }
        for _ in 0..count {
            stack.pop();
        }
        stack.push(Literal::Array(count));
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.drop_op {
        let _ = stack.pop();
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.ret {
        stack.clear();
        return TokenScanEvent::Continue;
    }
    if Some(op) == ops.syscall {
        if *pc + 4 > script.len() {
            return TokenScanEvent::Stop;
        }
        let hash = u32::from_le_bytes([
            script[*pc],
            script[*pc + 1],
            script[*pc + 2],
            script[*pc + 3],
        ]);
        *pc += 4;
        return TokenScanEvent::Syscall(hash);
    }

    if let Some(info) = crate::opcodes::lookup_by_byte(op) {
        let prefix_size = usize::from(info.operand_size_prefix);
        if prefix_size != 0 {
            let prefix_end = match pc.checked_add(prefix_size) {
                Some(end) if end <= script.len() => end,
                _ => return TokenScanEvent::Stop,
            };

            let dynamic_size = match prefix_size {
                1 => usize::from(script[*pc]),
                2 => usize::from(u16::from_le_bytes([script[*pc], script[*pc + 1]])),
                4 => u32::from_le_bytes([
                    script[*pc],
                    script[*pc + 1],
                    script[*pc + 2],
                    script[*pc + 3],
                ]) as usize,
                _ => return TokenScanEvent::Stop,
            };
            *pc = prefix_end;

            let operand_end = match pc
                .checked_add(usize::from(info.operand_size))
                .and_then(|end| end.checked_add(dynamic_size))
            {
                Some(end) if end <= script.len() => end,
                _ => return TokenScanEvent::Stop,
            };
            *pc = operand_end;
        } else if info.operand_size != 0 {
            let operand_end = match pc.checked_add(usize::from(info.operand_size)) {
                Some(end) if end <= script.len() => end,
                _ => return TokenScanEvent::Stop,
            };
            *pc = operand_end;
        }
    }

    stack.clear();
    TokenScanEvent::Continue
}
