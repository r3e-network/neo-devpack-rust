// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

mod decode;
mod literal;
mod opcodes;

use super::*;
use literal::Literal;
use opcodes::OpcodeBytes;

pub(crate) fn infer_contract_tokens(script: &[u8]) -> Vec<MethodToken> {
    use crate::syscalls;

    const MAX_TOKEN_METHOD_LEN: usize = 32;

    let ops = OpcodeBytes::collect();
    if ops.syscall.is_none() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let mut stack: Vec<Literal> = Vec::new();
    let mut pc = 0usize;
    while pc < script.len() {
        let op = script[pc];
        pc += 1;

        match decode::step(op, script, &mut pc, &ops, &mut stack) {
            decode::TokenScanEvent::Continue => {}
            decode::TokenScanEvent::Stop => break,
            decode::TokenScanEvent::Syscall(hash) => {
                if let Some(info) = syscalls::lookup_by_hash(hash) {
                    if info.name.eq_ignore_ascii_case("System.Contract.Call") {
                        let args = stack.pop().unwrap_or(Literal::Unknown);
                        let call_flags = stack.pop().unwrap_or(Literal::Unknown);
                        let method = stack.pop().unwrap_or(Literal::Unknown);
                        let hash_bytes = stack.pop().unwrap_or(Literal::Unknown);

                        if let (
                            Literal::Bytes(contract_hash),
                            Literal::Bytes(method_bytes),
                            Literal::Integer(flags),
                            Literal::Array(param_count),
                        ) = (
                            hash_bytes.clone(),
                            method.clone(),
                            call_flags.clone(),
                            args.clone(),
                        ) {
                            if contract_hash.len() == HASH160_LENGTH {
                                if let Ok(method_name) = String::from_utf8(method_bytes.clone()) {
                                    if method_name.len() > MAX_TOKEN_METHOD_LEN {
                                        continue;
                                    }
                                    if flags >= 0 && flags <= u8::MAX as i128 {
                                        let has_return_value = if pc < script.len() {
                                            Some(script[pc]) != ops.drop_op
                                        } else {
                                            true
                                        };
                                        if let Ok(parameters_count) = u16::try_from(param_count) {
                                            tokens.push(MethodToken {
                                                contract_hash: {
                                                    let mut array = [0u8; HASH160_LENGTH];
                                                    array.copy_from_slice(&contract_hash);
                                                    array
                                                },
                                                method: method_name,
                                                parameters_count,
                                                has_return_value,
                                                call_flags: flags as u8,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    } else if info.name.len() > MAX_TOKEN_METHOD_LEN {
                        stack.push(Literal::Unknown);
                    } else {
                        let has_return_value = if pc < script.len() {
                            Some(script[pc]) != ops.drop_op
                        } else {
                            true
                        };
                        tokens.push(MethodToken {
                            contract_hash: [0u8; HASH160_LENGTH],
                            method: info.name.to_string(),
                            parameters_count: 0,
                            has_return_value,
                            call_flags: 0,
                        });
                    }
                }

                stack.push(Literal::Unknown);
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::infer_contract_tokens;

    fn opcode(name: &str) -> u8 {
        crate::opcodes::lookup(name)
            .unwrap_or_else(|| panic!("missing opcode {name}"))
            .byte
    }

    #[test]
    fn infer_contract_tokens_skips_param_count_overflow() {
        let push0 = opcode("PUSH0");
        let pushint8 = opcode("PUSHINT8");
        let pushint32 = opcode("PUSHINT32");
        let pushdata1 = opcode("PUSHDATA1");
        let pack = opcode("PACK");
        let syscall = opcode("SYSCALL");

        let call_hash = crate::syscalls::lookup("System.Contract.Call")
            .expect("System.Contract.Call syscall exists")
            .hash;

        let mut script = Vec::new();

        script.push(pushdata1);
        script.push(20);
        script.extend(1u8..=20);

        script.push(pushdata1);
        script.push(4);
        script.extend_from_slice(b"ping");

        script.push(pushint8);
        script.push(5);

        script.extend(std::iter::repeat_n(push0, 70_000));

        script.push(pushint32);
        script.extend_from_slice(&70_000i32.to_le_bytes());
        script.push(pack);

        script.push(syscall);
        script.extend_from_slice(&call_hash.to_le_bytes());

        let tokens = infer_contract_tokens(&script);
        assert!(tokens.is_empty());
    }

    #[test]
    fn infer_contract_tokens_skips_param_count_that_overflows_usize() {
        let pushint8 = opcode("PUSHINT8");
        let pushint128 = opcode("PUSHINT128");
        let pushdata1 = opcode("PUSHDATA1");
        let pack = opcode("PACK");
        let syscall = opcode("SYSCALL");

        let call_hash = crate::syscalls::lookup("System.Contract.Call")
            .expect("System.Contract.Call syscall exists")
            .hash;

        let mut script = Vec::new();

        script.push(pushdata1);
        script.push(20);
        script.extend(1u8..=20);

        script.push(pushdata1);
        script.push(4);
        script.extend_from_slice(b"ping");

        script.push(pushint8);
        script.push(5);

        // 2^64 overflows usize on 64-bit targets if converted with `as`.
        let overflow_count = (u64::MAX as i128) + 1;
        script.push(pushint128);
        script.extend_from_slice(&overflow_count.to_le_bytes());
        script.push(pack);

        script.push(syscall);
        script.extend_from_slice(&call_hash.to_le_bytes());

        let tokens = infer_contract_tokens(&script);
        assert!(tokens.is_empty());
    }

    #[test]
    fn infer_contract_tokens_ignores_jump_operand_bytes() {
        let jmp_l = opcode("JMP_L");
        let syscall = opcode("SYSCALL");
        let runtime_log_hash = crate::syscalls::lookup("System.Runtime.Log")
            .expect("System.Runtime.Log syscall exists")
            .hash
            .to_le_bytes();

        let script = vec![
            jmp_l,
            syscall,
            runtime_log_hash[0],
            runtime_log_hash[1],
            runtime_log_hash[2],
            runtime_log_hash[3],
        ];

        let tokens = infer_contract_tokens(&script);
        assert!(tokens.is_empty());
    }
}
