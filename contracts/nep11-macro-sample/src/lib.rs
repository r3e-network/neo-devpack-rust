// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Integration test for the `nep11!` standard-library macro (L5).

use neo_devpack::nep11;

nep11! {
    contract MacroNep11Sample {
        symbol: "MCR",
        name: "MacroNep11Sample",
    }
}
