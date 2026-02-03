#!/bin/bash
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
#

wasmtime --invoke main -D logging=y -D debug-info=y -D address-map=y "$@" 0 0
