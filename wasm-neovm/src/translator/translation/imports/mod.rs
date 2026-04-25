// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, bail, Result};
use wasmparser::{FuncType, ValType};

use crate::adapters::ChainAdapter;
use crate::neo_syscalls;
use crate::opcodes;
use crate::syscalls;
use crate::translator::constants::*;
use crate::translator::helpers::*;
use crate::translator::runtime::{ensure_memory_access, RuntimeHelpers};
use crate::translator::types::StackValue;
use crate::translator::FunctionImport;

use super::features::FeatureTracker;

mod dispatch;
mod env;
mod opcode;
mod syscall;

pub(super) use dispatch::get_import_type_index;
pub(crate) use dispatch::handle_import_call;
pub(super) use env::try_handle_env_import;
pub(super) use syscall::try_handle_neo_import;
