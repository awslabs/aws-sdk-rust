#!/usr/bin/env python3
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

import json
import math
import os
import shlex
import subprocess
import sys
import unittest

DESIRED_WORKERS = 6


def main():
    if len(sys.argv) == 1:
        print(f"Usage: {sys.argv[0]} <rust versions to use...>")
        return 1

    rust_versions = sys.argv[1:]
    repository_root = get_cmd_output("git rev-parse --show-toplevel")
    os.chdir(repository_root)

    crate_paths = get_cmd_output("./tools/ci/list-sdk-crate-paths.sh").splitlines(keepends=False)
    batches = calculate_batches(len(crate_paths), DESIRED_WORKERS)

    output = {
        "crate_range": list(map(lambda b: f"{b[0]} {b[1]}", batches)),
        "rust_version": rust_versions
    }
    print(json.dumps(output))
    return 0


def calculate_batches(total_jobs, batch_count):
    batch_size = math.ceil(total_jobs / batch_count)
    return list(map(lambda i: (i, min(total_jobs, i + batch_size)), range(0, total_jobs, batch_size)))


def get_cmd_output(command):
    result = subprocess.run(shlex.split(command), capture_output=True, check=True)
    return result.stdout.decode("utf-8").strip()


class SelfTest(unittest.TestCase):
    def test_calculate_batches(self):
        self.assertEqual([(0, 2), (2, 4), (4, 5)], calculate_batches(5, 3))
        self.assertEqual([(0, 2), (2, 4), (4, 6)], calculate_batches(6, 3))
        self.assertEqual([(0, 56), (56, 111)], calculate_batches(111, 2))
        self.assertEqual([(0, 111)], calculate_batches(111, 1))
        self.assertEqual([(0, 1), (1, 2)], calculate_batches(2, 10))


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        sys.argv.pop()
        unittest.main()
    else:
        sys.exit(main())
