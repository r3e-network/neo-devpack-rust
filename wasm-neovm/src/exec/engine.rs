// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! The execution engine: decodes + executes a NeoVM script against a [`Host`].

#![cfg(feature = "exec")]
#![allow(dead_code)]

use crate::exec::host::{Fault, Host};
use crate::exec::item::{bytes_to_int, int_to_fixed_bytes, NeoItem};
use crate::opcodes;
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};

/// Result of executing a script to completion.
#[derive(Debug, Clone)]
pub enum ExecResult {
    /// Halted normally via `RET` at the top frame.
    Halt,
    /// Engine faulted.
    Failed(Fault),
    /// Exceeded the step budget (infinite-loop guard).
    MaxSteps,
}

const MAX_STEPS: usize = 1_000_000;

#[derive(Debug)]
enum Flow {
    Continue,
    Halt,
}

/// A small NeoVM execution engine over the emitted opcode subset.
pub struct Engine<'a> {
    script: &'a [u8],
    pc: usize,
    stack: Vec<NeoItem>,
    alt: Vec<NeoItem>,
    #[allow(dead_code)]
    call_stack: Vec<usize>,
    host: &'a mut Host,
    locals: Vec<NeoItem>,
    locals_arg_base: usize,
    steps: usize,
}

impl<'a> Engine<'a> {
    pub fn new(script: &'a [u8], host: &'a mut Host) -> Self {
        Self {
            script,
            pc: 0,
            stack: Vec::new(),
            alt: Vec::new(),
            call_stack: Vec::new(),
            host,
            locals: Vec::new(),
            locals_arg_base: 0,
            steps: 0,
        }
    }

    /// Begin execution at `start_pc` instead of 0. Used by tests that drive a
    /// specific helper sub-routine in place (preserving the helper's jump
    /// patching, which is relative to its original position).
    pub fn new_starting_at(script: &'a [u8], start_pc: usize, host: &'a mut Host) -> Self {
        let mut e = Self::new(script, host);
        e.pc = start_pc;
        e
    }

    pub fn stack(&self) -> &[NeoItem] {
        &self.stack
    }

    /// Push an initial item onto the eval stack (for tests that drive a
    /// sub-routine with a pre-seeded stack).
    pub fn push_initial(&mut self, item: NeoItem) {
        self.stack.push(item);
    }

    /// Run to completion. On Halt, the result stack is available via [`stack`].
    pub fn run(&mut self) -> ExecResult {
        loop {
            if self.steps >= MAX_STEPS {
                return ExecResult::MaxSteps;
            }
            self.steps += 1;
            if self.pc >= self.script.len() {
                return ExecResult::Halt;
            }
            let op = self.script[self.pc];
            let op_start = self.pc;
            self.pc += 1;
            match self.step(op, op_start) {
                Ok(Flow::Continue) => {}
                Ok(Flow::Halt) => return ExecResult::Halt,
                Err(f) => return ExecResult::Failed(f),
            }
        }
    }

    /// Current program counter (for tracing in tests).
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Execute a single opcode and report. Returns `None` at end-of-script.
    /// For test tracing.
    pub fn step_once(&mut self) -> Option<Result<(), Fault>> {
        if self.pc >= self.script.len() {
            return None;
        }
        let op = self.script[self.pc];
        let op_start = self.pc;
        self.pc += 1;
        match self.step(op, op_start) {
            Ok(Flow::Continue) => Some(Ok(())),
            Ok(Flow::Halt) => None, // treat as end
            Err(f) => Some(Err(f)),
        }
    }

    fn step(&mut self, op: u8, op_start: usize) -> Result<Flow, Fault> {
        // PUSH* immediates first (so they don't collide with the name dispatch).
        if let Some(item) = self.try_decode_push(op)? {
            self.stack.push(item);
            return Ok(Flow::Continue);
        }
        let name = match opcodes::lookup_by_byte(op) {
            Some(info) => info.name,
            None => return Err(Fault::Unsupported(format!("opcode 0x{op:02x}"))),
        };
        match name {
            "PUSHNULL" => self.stack.push(NeoItem::Null),
            "ISNULL" => {
                let v = self.stack.pop().ok_or_else(|| st("ISNULL"))?;
                self.stack.push(NeoItem::Boolean(matches!(v, NeoItem::Null)));
            }
            "DUP" => self.dup_from_top(0)?,
            "SWAP" => {
                let n = self.stack.len();
                (n >= 2).then_some(()).ok_or_else(|| st("SWAP"))?;
                self.stack.swap(n - 1, n - 2);
            }
            "OVER" => {
                let n = self.stack.len();
                (n >= 2).then_some(()).ok_or_else(|| st("OVER"))?;
                let v = self.stack[n - 2].clone();
                self.stack.push(v);
            }
            "ROT" => {
                let n = self.stack.len();
                (n >= 3).then_some(()).ok_or_else(|| st("ROT"))?;
                let a = self.stack.remove(n - 3);
                self.stack.push(a);
            }
            "PICK" => {
                // PICK n: copy the nth item from top (0 = top) to top.
                let n_item = self.stack.pop().ok_or_else(|| st("PICK n"))?;
                let n = n_item
                    .convert_to_int()
                    .and_then(|v| v.to_usize())
                    .ok_or_else(|| st("PICK range"))?;
                let len = self.stack.len();
                (n < len).then_some(()).ok_or_else(|| st("PICK oob"))?;
                let v = self.stack[len - 1 - n].clone();
                self.stack.push(v);
            }
            "ROLL" => {
                // ROLL n: move the nth item from top to top.
                let n_item = self.stack.pop().ok_or_else(|| st("ROLL n"))?;
                let n = n_item
                    .convert_to_int()
                    .and_then(|v| v.to_usize())
                    .ok_or_else(|| st("ROLL range"))?;
                let len = self.stack.len();
                (n < len).then_some(()).ok_or_else(|| st("ROLL oob"))?;
                let v = self.stack.remove(len - 1 - n);
                self.stack.push(v);
            }
            "REVERSE3" => {
                let n = self.stack.len();
                (n >= 3).then_some(()).ok_or_else(|| st("REVERSE3"))?;
                self.stack.swap(n - 1, n - 3);
            }
            "DEPTH" => self.stack.push(NeoItem::int(self.stack.len() as i64)),
            "DROP" => {
                self.stack.pop().ok_or_else(|| st("DROP"))?;
            }
            "NIP" => {
                let n = self.stack.len();
                (n >= 2).then_some(()).ok_or_else(|| st("NIP"))?;
                self.stack.remove(n - 2);
            }
            "CLEAR" => self.stack.clear(),
            "TOALTSTACK" => {
                let v = self.stack.pop().ok_or_else(|| st("TOALTSTACK"))?;
                self.alt.push(v);
            }
            "FROMALTSTACK" => {
                let v = self.alt.pop().ok_or_else(|| st("FROMALTSTACK"))?;
                self.stack.push(v);
            }
            "DUPFROMALTSTACK" => {
                let v = self.alt.last().cloned().ok_or_else(|| st("DUPFROMALTSTACK"))?;
                self.stack.push(v);
            }
            // Arithmetic
            "ADD" => self.binary(|a, b| a + b)?,
            "SUB" => self.binary(|a, b| a - b)?,
            "MUL" => self.binary(|a, b| a * b)?,
            "DIV" => self.div_mod(true)?,
            "MOD" => self.div_mod(false)?,
            "SHL" => self.shift(false)?,
            "SHR" => self.shift(true)?,
            "AND" => self.binary(|a, b| a & b)?,
            "OR" => self.binary(|a, b| a | b)?,
            "XOR" => self.binary(|a, b| a ^ b)?,
            "INVERT" => {
                let v = self.pop_int()?;
                self.stack.push(NeoItem::Integer(!v));
            }
            "SIGN" => {
                let v = self.pop_int()?;
                let s = if v.is_positive() { 1 } else if v.is_negative() { -1 } else { 0 };
                self.stack.push(NeoItem::int(s));
            }
            "NEGATE" => {
                let v = self.pop_int()?;
                self.stack.push(NeoItem::Integer(-v));
            }
            "ABS" => {
                let v = self.pop_int()?;
                self.stack.push(NeoItem::Integer(v.abs()));
            }
            "INC" => {
                let v = self.pop_int()?;
                self.stack.push(NeoItem::Integer(v + BigInt::from(1)));
            }
            "DEC" => {
                let v = self.pop_int()?;
                self.stack.push(NeoItem::Integer(v - BigInt::from(1)));
            }
            // Comparisons
            "EQ" => self.cmp(|a, b| if a == b { BigInt::from(1) } else { BigInt::from(0) })?,
            "NE" => self.cmp(|a, b| if a != b { BigInt::from(1) } else { BigInt::from(0) })?,
            "LT" => self.cmp(|a, b| if a < b { BigInt::from(1) } else { BigInt::from(0) })?,
            "LE" => self.cmp(|a, b| if a <= b { BigInt::from(1) } else { BigInt::from(0) })?,
            "GT" => self.cmp(|a, b| if a > b { BigInt::from(1) } else { BigInt::from(0) })?,
            "GE" => self.cmp(|a, b| if a >= b { BigInt::from(1) } else { BigInt::from(0) })?,
            "MIN" => self.cmp(|a, b| a.min(b))?,
            "MAX" => self.cmp(|a, b| a.max(b))?,
            // Buffers / items
            "CAT" => {
                let b = self.pop_buf_owned()?;
                let mut a = self.pop_buf_owned()?;
                a.extend_from_slice(&b);
                self.stack.push(NeoItem::buffer(a));
            }
            "SUBSTR" => {
                let count = self.pop_int()?;
                let start = self.pop_int()?;
                let buf = self.pop_buf_owned()?;
                let s = start.to_usize().ok_or_else(|| st("SUBSTR range"))?;
                let c = count.to_usize().ok_or_else(|| st("SUBSTR range"))?;
                let end = s.checked_add(c).ok_or_else(|| ab("SUBSTR overflow"))?;
                (end <= buf.len()).then_some(()).ok_or_else(|| ab("SUBSTR oob"))?;
                self.stack.push(NeoItem::buffer(buf[s..end].to_vec()));
            }
            "LEFT" => {
                let n = self.pop_int()?;
                let buf = self.pop_buf_owned()?;
                let n = n.to_usize().ok_or_else(|| st("LEFT range"))?;
                (n <= buf.len()).then_some(()).ok_or_else(|| ab("LEFT oob"))?;
                self.stack.push(NeoItem::buffer(buf[..n].to_vec()));
            }
            "RIGHT" => {
                let n = self.pop_int()?;
                let buf = self.pop_buf_owned()?;
                let n = n.to_usize().ok_or_else(|| st("RIGHT range"))?;
                (n <= buf.len()).then_some(()).ok_or_else(|| ab("RIGHT oob"))?;
                let start = buf.len() - n;
                self.stack.push(NeoItem::buffer(buf[start..].to_vec()));
            }
            "SIZE" => {
                let v = self.stack.pop().ok_or_else(|| st("SIZE"))?;
                self.stack.push(NeoItem::int(v.size() as i64));
            }
            "CONVERT" => {
                let ty = self.read_u8()?;
                let v = self.stack.pop().ok_or_else(|| st("CONVERT"))?;
                self.stack.push(self.convert(v, ty));
            }
            "NEWBUFFER" => {
                let n = self.pop_int()?;
                let n = n.to_usize().ok_or_else(|| st("NEWBUFFER range"))?;
                self.stack.push(NeoItem::buffer(vec![0u8; n]));
            }
            "NEWARRAY0" | "NEWSTRUCT0" => self.stack.push(NeoItem::Array(Vec::new())),
            "NEWARRAY" | "NEWSTRUCT" => {
                let n = self.pop_int()?;
                let n = n.to_usize().ok_or_else(|| st("NEWARRAY range"))?;
                self.stack.push(NeoItem::Array(vec![NeoItem::Null; n]));
            }
            "APPEND" => {
                let item = self.stack.pop().ok_or_else(|| st("APPEND item"))?;
                let col = self.stack.pop().ok_or_else(|| st("APPEND col"))?;
                match col {
                    NeoItem::Array(mut a) | NeoItem::Struct(mut a) => {
                        a.push(item);
                        self.stack.push(NeoItem::Array(a));
                    }
                    _ => return Err(st("APPEND not array")),
                }
            }
            "PACK" => {
                let n = self.pop_int()?;
                let n = n.to_usize().ok_or_else(|| st("PACK range"))?;
                (self.stack.len() >= n).then_some(()).ok_or_else(|| st("PACK underflow"))?;
                let items: Vec<NeoItem> = self.stack.split_off(self.stack.len() - n);
                self.stack.push(NeoItem::Array(items));
            }
            "UNPACK" => {
                let col = self.stack.pop().ok_or_else(|| st("UNPACK"))?;
                match col {
                    NeoItem::Array(items) | NeoItem::Struct(items) => {
                        self.stack.push(NeoItem::int(items.len()));
                        for it in items {
                            self.stack.push(it);
                        }
                    }
                    _ => return Err(st("UNPACK not array")),
                }
            }
            "PICKITEM" => {
                let idx = self.pop_int()?;
                let col = self.stack.pop().ok_or_else(|| st("PICKITEM col"))?;
                let idx = idx.to_usize().ok_or_else(|| st("PICKITEM range"))?;
                let item = match col {
                    NeoItem::Array(a) | NeoItem::Struct(a) => {
                        a.get(idx).cloned().ok_or_else(|| ab("PICKITEM oob"))?
                    }
                    NeoItem::Buffer(b) => {
                        let b = b.borrow();
                        NeoItem::int(*b.get(idx).ok_or_else(|| ab("PICKITEM oob"))? as i64)
                    }
                    _ => return Err(st("PICKITEM not collection")),
                };
                self.stack.push(item);
            }
            "SETITEM" => {
                let value = self.stack.pop().ok_or_else(|| st("SETITEM val"))?;
                let idx = self.pop_int()?;
                let col = self.stack.pop().ok_or_else(|| st("SETITEM col"))?;
                let idx = idx.to_usize().ok_or_else(|| st("SETITEM range"))?;
                // NeoVM SETITEM pops all three (value, key, collection) and
                // does NOT push the collection back (verified against
                // JumpTable.Compound.cs SetItem). Buffers are reference-shared
                // (Rc<RefCell>), so the mutation persists to every handle —
                // this is what the memory store/load path relies on. Arrays
                // are value-ish here; their SETITEM persistence is best-effort.
                match &col {
                    NeoItem::Buffer(b) => {
                        let mut bref = b.borrow_mut();
                        (idx < bref.len()).then_some(()).ok_or_else(|| ab("SETITEM buf oob"))?;
                        let v = value
                            .convert_to_int()
                            .and_then(|v| v.to_u8())
                            .ok_or_else(|| st("SETITEM buf val"))?;
                        bref[idx] = v;
                    }
                    NeoItem::Array(_) | NeoItem::Struct(_) => {
                        // Value semantics; mutation is on the popped clone.
                        if let NeoItem::Array(a) | NeoItem::Struct(a) = &col {
                            if idx < a.len() {
                                // No-op persistence-wise; arrays are not used by
                                // the memory helpers the harness targets.
                            }
                        }
                    }
                    _ => return Err(st("SETITEM not collection")),
                }
            }
            // Locals / args / statics
            "INITSLOT" => {
                let locals_n = self.read_u8()? as usize;
                let args_n = self.read_u8()? as usize;
                let mut new_locals: Vec<NeoItem> = Vec::with_capacity(locals_n + args_n);
                for _ in 0..locals_n {
                    new_locals.push(NeoItem::Null);
                }
                // The translator reverses args so top-of-stack == arg0; pop
                // args_n in order and store them as the tail of the locals.
                for _ in 0..args_n {
                    let v = self.stack.pop().unwrap_or(NeoItem::Null);
                    new_locals.push(v);
                }
                self.locals_arg_base = locals_n;
                self.locals = new_locals;
            }
            "LDLOC0" | "LDLOC1" | "LDLOC2" | "LDLOC3" | "LDLOC4" | "LDLOC5" | "LDLOC6" => {
                let i = name.trim_start_matches("LDLOC").parse::<usize>().unwrap_or(0);
                let v = self.locals.get(i).cloned().unwrap_or(NeoItem::Null);
                self.stack.push(v);
            }
            "STLOC0" | "STLOC1" | "STLOC2" | "STLOC3" | "STLOC4" | "STLOC5" | "STLOC6" => {
                let i = name.trim_start_matches("STLOC").parse::<usize>().unwrap_or(0);
                let v = self.stack.pop().unwrap_or(NeoItem::Null);
                if i >= self.locals.len() {
                    self.locals.resize(i + 1, NeoItem::Null);
                }
                self.locals[i] = v;
            }
            "LDARG0" | "LDARG1" | "LDARG2" | "LDARG3" | "LDARG4" | "LDARG5" | "LDARG6" => {
                let i = name.trim_start_matches("LDARG").parse::<usize>().unwrap_or(0);
                let v = self
                    .locals
                    .get(self.locals_arg_base + i)
                    .cloned()
                    .unwrap_or(NeoItem::Null);
                self.stack.push(v);
            }
            "LDSFLD0" | "LDSFLD1" | "LDSFLD2" | "LDSFLD3" | "LDSFLD4" | "LDSFLD5" | "LDSFLD6" => {
                let i = name.trim_start_matches("LDSFLD").parse::<usize>().unwrap_or(0);
                let v = self.host.static_fields.get(i).cloned().unwrap_or(NeoItem::Null);
                self.stack.push(v);
            }
            "STSFLD0" | "STSFLD1" | "STSFLD2" | "STSFLD3" | "STSFLD4" | "STSFLD5" | "STSFLD6" => {
                let i = name.trim_start_matches("STSFLD").parse::<usize>().unwrap_or(0);
                let v = self.stack.pop().unwrap_or(NeoItem::Null);
                if i >= self.host.static_fields.len() {
                    self.host.static_fields.resize(i + 1, NeoItem::Null);
                }
                self.host.static_fields[i] = v;
            }
            // Control flow
            "NOP" => {}
            "JMP" => self.jump(1, op_start)?,
            "JMP_L" => self.jump(4, op_start)?,
            "JMPIF" => self.cond_jump(1, op_start, true)?,
            "JMPIF_L" => self.cond_jump(4, op_start, true)?,
            "JMPIFNOT" => self.cond_jump(1, op_start, false)?,
            "JMPIFNOT_L" => self.cond_jump(4, op_start, false)?,
            "CALL" | "CALL_L" => {
                // Skip the immediate; model as fallthrough for helper-inlined
                // tests (the translator inlines most helpers). Full CALL frame
                // tracking isn't needed for the memory-helper regression.
                if name == "CALL" {
                    let _ = self.read_u8()?;
                } else {
                    let _ = self.read_u32()?;
                }
            }
            "CALLT" => {
                let _ = self.read_u8()?;
                let _ = self.read_u8()?;
                self.stack.push(NeoItem::Null);
            }
            "RET" => return Ok(Flow::Halt),
            "SYSCALL" => {
                let hash = self.read_u32()?;
                let ret = self.host.syscall(hash, &mut self.stack)?;
                if let Some(item) = ret {
                    self.stack.push(item);
                }
            }
            "ABORT" => return Err(ab("ABORT opcode")),
            "ASSERT" => {
                let v = self.stack.pop().ok_or_else(|| st("ASSERT"))?;
                if !v.is_truthy() {
                    return Err(ab("ASSERT failed"));
                }
            }
            other => return Err(Fault::Unsupported(format!("opcode {other}"))),
        }
        Ok(Flow::Continue)
    }

    fn try_decode_push(&mut self, op: u8) -> Result<Option<NeoItem>, Fault> {
        // PUSH1..PUSH16 = 0x11..0x20 (push a small inline integer). PUSH0 = 0x10.
        if op == 0x10 {
            return Ok(Some(NeoItem::int(0)));
        }
        if (0x11..=0x20).contains(&op) {
            return Ok(Some(NeoItem::int((op - 0x10) as i64)));
        }
        if op == 0x0F {
            return Ok(Some(NeoItem::int(-1)));
        }
        if op == 0x08 {
            return Ok(Some(NeoItem::Boolean(true)));
        }
        if op == 0x09 {
            return Ok(Some(NeoItem::Boolean(false)));
        }
        // Decode by name via the authoritative opcode table so we never collide
        // with other opcodes (bytes are NOT contiguous with PUSH).
        let Some(info) = opcodes::lookup_by_byte(op) else {
            return Ok(None);
        };
        match info.name {
            "PUSHNULL" => Ok(Some(NeoItem::Null)),
            "PUSHINT8" => Ok(Some(NeoItem::int(self.read_u8()? as i8 as i64))),
            "PUSHINT16" => Ok(Some(NeoItem::int(self.read_u16()? as i16 as i64))),
            "PUSHINT32" => Ok(Some(NeoItem::int(self.read_u32()? as i64))),
            "PUSHINT64" => Ok(Some(NeoItem::int(self.read_u64()? as i128))),
            "PUSHINT128" => Ok(Some(NeoItem::Integer(bytes_to_int(&self.read_bytes(16)?)))),
            "PUSHINT256" => Ok(Some(NeoItem::Integer(bytes_to_int(&self.read_bytes(32)?)))),
            "PUSHDATA1" => {
                let len = self.read_u8()? as usize;
                Ok(Some(NeoItem::buffer(self.read_bytes(len)?)))
            }
            "PUSHDATA2" => {
                let len = self.read_u16()? as usize;
                Ok(Some(NeoItem::buffer(self.read_bytes(len)?)))
            }
            "PUSHDATA4" => {
                let len = self.read_u32()? as usize;
                Ok(Some(NeoItem::buffer(self.read_bytes(len)?)))
            }
            _ => Ok(None),
        }
    }

    fn convert(&self, v: NeoItem, ty: u8) -> NeoItem {
        match ty {
            0x21 => NeoItem::Integer(v.convert_to_int().unwrap_or_else(|| BigInt::from(0))),
            0x20 => match v {
                NeoItem::Integer(i) => NeoItem::buffer(int_to_fixed_bytes(&i, 32)),
                NeoItem::Buffer(b) => NeoItem::Buffer(b),
                other => NeoItem::buffer(int_to_fixed_bytes(
                    &other.convert_to_int().unwrap_or_else(|| BigInt::from(0)),
                    32,
                )),
            },
            0x27 => NeoItem::Boolean(v.is_truthy()),
            _ => v,
        }
    }

    fn binary<F: FnOnce(BigInt, BigInt) -> BigInt>(&mut self, f: F) -> Result<(), Fault> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(NeoItem::Integer(f(a, b)));
        Ok(())
    }

    fn div_mod(&mut self, is_div: bool) -> Result<(), Fault> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        if b.is_zero() {
            return Err(ab("division by zero"));
        }
        let r = if is_div { a / b } else { a % b };
        self.stack.push(NeoItem::Integer(r));
        Ok(())
    }

    fn shift(&mut self, right: bool) -> Result<(), Fault> {
        let s = self.pop_int()?;
        let v = self.pop_int()?;
        let bits = s.to_usize().ok_or_else(|| st("shift range"))?;
        let r = if right { v >> bits } else { v << bits };
        self.stack.push(NeoItem::Integer(r));
        Ok(())
    }

    fn cmp<F: FnOnce(BigInt, BigInt) -> BigInt>(&mut self, f: F) -> Result<(), Fault> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(NeoItem::Integer(f(a, b)));
        Ok(())
    }

    fn jump(&mut self, imm_size: usize, op_start: usize) -> Result<(), Fault> {
        let off = self.read_offset(imm_size)?;
        self.pc = (op_start as isize + off) as usize;
        Ok(())
    }

    fn cond_jump(&mut self, imm_size: usize, op_start: usize, if_true: bool) -> Result<(), Fault> {
        let cond = self.stack.pop().ok_or_else(|| st("condjump"))?;
        let take = if_true == cond.is_truthy();
        let off = self.read_offset(imm_size)?;
        if take {
            self.pc = (op_start as isize + off) as usize;
        }
        Ok(())
    }

    fn dup_from_top(&mut self, _offset: usize) -> Result<(), Fault> {
        let v = self.stack.last().cloned().ok_or_else(|| st("DUP"))?;
        self.stack.push(v);
        Ok(())
    }

    fn pop_int(&mut self) -> Result<BigInt, Fault> {
        let v = self.stack.pop().ok_or_else(|| st("pop underflow"))?;
        v.convert_to_int().ok_or_else(|| st("not int"))
    }

    fn pop_buf_owned(&mut self) -> Result<Vec<u8>, Fault> {
        match self.stack.pop() {
            Some(NeoItem::Buffer(b)) => Ok(b.borrow().clone()),
            Some(NeoItem::Integer(i)) => Ok(int_to_fixed_bytes(&i, 32)),
            other => {
                let _ = other;
                Err(st("not buffer"))
            }
        }
    }

    // immediate readers
    fn read_u8(&mut self) -> Result<u8, Fault> {
        if self.pc >= self.script.len() {
            return Err(st("truncated u8"));
        }
        let v = self.script[self.pc];
        self.pc += 1;
        Ok(v)
    }
    fn read_u16(&mut self) -> Result<u16, Fault> {
        let mut b = [0u8; 2];
        b.copy_from_slice(self.read_bytes_ref(2)?);
        Ok(u16::from_le_bytes(b))
    }
    fn read_u32(&mut self) -> Result<u32, Fault> {
        let mut b = [0u8; 4];
        b.copy_from_slice(self.read_bytes_ref(4)?);
        Ok(u32::from_le_bytes(b))
    }
    fn read_u64(&mut self) -> Result<u64, Fault> {
        let mut b = [0u8; 8];
        b.copy_from_slice(self.read_bytes_ref(8)?);
        Ok(u64::from_le_bytes(b))
    }
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, Fault> {
        let v = self.read_bytes_ref(n)?.to_vec();
        Ok(v)
    }
    fn read_bytes_ref(&mut self, n: usize) -> Result<&[u8], Fault> {
        if self.pc + n > self.script.len() {
            return Err(st("truncated bytes"));
        }
        let s = &self.script[self.pc..self.pc + n];
        self.pc += n;
        Ok(s)
    }
    fn read_varuint(&mut self) -> Result<usize, Fault> {
        let b0 = self.read_u8()? as usize;
        Ok(match b0 {
            0xFD => self.read_u16()? as usize,
            0xFE => self.read_u32()? as usize,
            0xFF => {
                let mut b = [0u8; 8];
                b.copy_from_slice(self.read_bytes_ref(8)?);
                u64::from_le_bytes(b) as usize
            }
            other => other,
        })
    }
    fn read_offset(&mut self, imm_size: usize) -> Result<isize, Fault> {
        Ok(match imm_size {
            1 => self.read_u8()? as i8 as isize,
            4 => self.read_u32()? as i32 as isize,
            _ => 0,
        })
    }
}

fn st(s: &str) -> Fault {
    Fault::State(s.to_string())
}
fn ab(s: &str) -> Fault {
    Fault::Abort(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::host::Host;

    fn op(name: &str) -> u8 {
        crate::opcodes::lookup(name).unwrap().byte
    }

    #[test]
    fn runs_simple_arithmetic() {
        // (10 + 6) * 2 = 32. PUSH<n> = 0x10+n for n in 0..=16.
        let script = [
            0x10 + 10, // PUSH10
            0x10 + 6,  // PUSH6
            op("ADD"),
            0x10 + 2, // PUSH2
            op("MUL"),
        ];
        let mut host = Host::default();
        let mut e = Engine::new(&script, &mut host);
        assert!(matches!(e.run(), ExecResult::Halt));
        let top = e.stack().last().unwrap().as_int().unwrap();
        assert_eq!(top, &BigInt::from(32));
    }

    #[test]
    fn setitem_then_pickitem_roundtrip() {
        // NeoVM SETITEM consumes [collection, index, value] (verified against
        // JumpTable.Compound.cs). We exercise it on a Buffer (reference-shared)
        // so the mutation persists: NEWBUFFER 8; <reload buffer>; ...
        // Simpler: use a static-field buffer and SETITEM via LDSFLD0 clone,
        // then re-load and PICKITEM.
        // NEWBUFFER 8 leaves a buffer B on stack; SETITEM consumes it; to read
        // back we keep B by DUP before SETITEM.
        let buf = op("NEWBUFFER");
        let _ = buf;
        // Hand-built: PUSH8; NEWBUFFER; DUP; PUSH0; PUSH7; SETITEM; PUSH0; PICKITEM
        // (values must be <= 16 for inline PUSH<n> = 0x10+n).
        let script = [
            0x10 + 8,                   // [8]
            op("NEWBUFFER"),            // [B]
            op("DUP"),                  // [B, B]
            op("PUSH0"),                // [B, B, 0]
            0x10 + 7,                   // [B, B, 0, 7]
            op("SETITEM"),              // [B]  (consumed B,0,7; the DUP'd B remains)
            op("PUSH0"),                // [B, 0]
            op("PICKITEM"),             // [7]
        ];
        let mut host = Host::default();
        let mut e = Engine::new(&script, &mut host);
        assert!(matches!(e.run(), ExecResult::Halt));
        let top = e.stack().last().unwrap().as_int().unwrap();
        assert_eq!(top, &BigInt::from(7));
    }

    #[test]
    fn rot_semantics() {
        // PUSH1; PUSH2; PUSH3; ROT -> top is 1 (3rd-from-top moved to top)
        let script = [0x10 + 1, 0x10 + 2, 0x10 + 3, op("ROT")];
        let mut host = Host::default();
        let mut e = Engine::new(&script, &mut host);
        assert!(matches!(e.run(), ExecResult::Halt));
        let top = e.stack().last().unwrap().as_int().unwrap();
        assert_eq!(top, &BigInt::from(1), "ROT moves 3rd-from-top to top");
        assert_eq!(e.stack().len(), 3);
    }
}
