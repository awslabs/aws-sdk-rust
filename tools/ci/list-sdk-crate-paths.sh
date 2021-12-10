#!/bin/bash
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

set -e
cd $(git rev-parse --show-toplevel)

for directory in $(ls -d sdk/*); do
    if [[ -f "${directory}/Cargo.toml" ]]; then echo "${directory}"; fi
done

for directory in $(ls -d examples/*); do
    if [[ -f "${directory}/Cargo.toml" ]]; then echo "${directory}"; fi
done