#!/usr/bin/env python3
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

import sys
import os
import subprocess
import shlex


def main():
    if len(sys.argv) < 4:
        print(f"Usage: {sys.argv[0]} <range-start-inclusive> <range-end-exclusive> <cmd>")
        return 1

    range_start_inclusive = int(sys.argv[1])
    range_end_exclusive = int(sys.argv[2])
    cmd = sys.argv[3:]

    if range_end_exclusive <= range_start_inclusive or range_end_exclusive < 0 or range_start_inclusive < 0:
        print("Invalid range")
        return 1

    repository_root = get_cmd_output("git rev-parse --show-toplevel")
    os.chdir(repository_root)

    crate_paths = get_cmd_output("./tools/ci/list-sdk-crate-paths.sh").splitlines(keepends=False)
    if range_start_inclusive >= len(crate_paths):
        print("Range start is invalid")
        return 1
    if range_end_exclusive > len(crate_paths):
        print("Range end is invalid")
        return 1

    crate_paths = crate_paths[range_start_inclusive:range_end_exclusive]
    for crate_path in crate_paths:
        os.chdir(f"{repository_root}/{crate_path}")
        subprocess.run(cmd, check=True)

    return 0


def get_cmd_output(command):
    result = subprocess.run(shlex.split(command), capture_output=True, check=True)
    return result.stdout.decode("utf-8").strip()


if __name__ == "__main__":
    sys.exit(main())
