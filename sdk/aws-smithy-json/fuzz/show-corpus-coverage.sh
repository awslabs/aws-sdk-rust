#!/bin/bash
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
#

# Script that gathers coverage from the entire fuzz corpus and shows covered lines in the terminal.
set -ex

# Run instrumented json_deserialize_corpus_cov to run the entire fuzz corpus with coverage
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE=coverage.profraw cargo run --release --bin json_deserialize_corpus_cov

# Convert raw coverage into profdata
cargo profdata -- merge -sparse coverage.profraw -o coverage.profdata

# Show coverage
cargo cov -- show --use-color --ignore-filename-regex='/.cargo/registry' --instr-profile=coverage.profdata --object target/release/json_deserialize_corpus_cov --show-instantiations --show-line-counts-or-regions | less -R
