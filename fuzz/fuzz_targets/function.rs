// SPDX-FileCopyrightText: Copyright The arm-psci Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Convert 4 registers into `Function` then back. Check if the register values match.

#![no_main]

use arm_psci::Function;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u64; 4]| {
    let Ok(func) = Function::try_from(&data) else {
        return;
    };

    func.function_id();
    let mut out = [0u64; 4];
    func.copy_to_array(&mut out);

    // MemProtect has a boolean which breaks the below test
    if matches!(func, Function::MemProtect { .. }) {
        return;
    }
    for (reg_in, reg_out) in data.iter().zip(out) {
        assert_eq!(reg_in & reg_out, reg_out, "Register mismatch: {:?}", func);
    }
});
