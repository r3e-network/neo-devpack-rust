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
                                        tokens.push(MethodToken {
                                            contract_hash: {
                                                let mut array = [0u8; HASH160_LENGTH];
                                                array.copy_from_slice(&contract_hash);
                                                array
                                            },
                                            method: method_name,
                                            parameters_count: param_count as u16,
                                            has_return_value,
                                            call_flags: flags as u8,
                                        });
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
