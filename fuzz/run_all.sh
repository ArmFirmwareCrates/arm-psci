#!/bin/sh -e
# SPDX-FileCopyrightText: Copyright The arm-psci Contributors.
# SPDX-License-Identifier: MIT OR Apache-2.0

cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=nightly

for fuzzer in $(cargo fuzz list); do
	cargo fuzz run "$fuzzer" -- "${@:--runs=10000000}" &
done
wait
