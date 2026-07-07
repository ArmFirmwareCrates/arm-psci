#!/bin/sh -xe
# SPDX-FileCopyrightText: Copyright The arm-psci Contributors.
# SPDX-License-Identifier: MIT OR Apache-2.0

cd "$(dirname "$0")/.."
export RUSTUP_TOOLCHAIN=nightly
triple=$(rustc -vV | sed -n 's/^host: //p')
target_dir=target/$triple/coverage/$triple/release
mkdir -p fuzz/lcov

for fuzzer in $(cargo fuzz list); do
	cargo fuzz coverage "$fuzzer"
	rust-cov export "$target_dir/$fuzzer" --format=lcov -instr-profile="fuzz/coverage/$fuzzer/coverage.profdata" >"fuzz/lcov/single-$fuzzer.info"
done

cd fuzz/lcov
lcov $(printf ' -a %s' single-*.info) -o merged.info
lcov -e merged.info '*arm-psci*' --rc branch_coverage=1 -o filtered.info
genhtml filtered.info
