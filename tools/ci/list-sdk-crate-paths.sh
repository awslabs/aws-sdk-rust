#!/bin/bash
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

# This script lists all of the crates in this repository that are
# considered to be a part of the SDK distribution. This does not include
# tools, such as the publisher tool, since these are not intended to be
# used by customers of the SDK, and also have their own CI.

set -e
cd $(git rev-parse --show-toplevel)

for directory in $(ls -d sdk/*); do
    if [[ -f "${directory}/Cargo.toml" ]]; then echo "${directory}"; fi
done

for directory in $(ls -d examples/*); do
    if [[ -f "${directory}/Cargo.toml" ]]; then echo "${directory}"; fi
done