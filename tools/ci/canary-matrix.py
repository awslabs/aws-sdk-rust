#!/usr/bin/env -S python3 -u
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

# This script is used by the canary CI to determine the versions of the SDK
# to run the canary against.

import argparse
import json
import subprocess
import sys
import unittest


def get_sdk_versions(desired_versions_count, aws_config_metadata_lines):
    versions = []
    for line in aws_config_metadata_lines:
        parsed = json.loads(line)
        if parsed["yanked"] != True:
            versions.append(parsed["vers"])
    versions = versions[-desired_versions_count:]
    return versions


# Reads the crates.io index to determine the latest `desired_versions_count` versions of the AWS SDK
def acquire_sdk_versions(desired_versions_count):
    result = subprocess.run(
        ["curl", "--fail", "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/aw/s-/aws-config"],
        check=True,
        capture_output=True
    )
    aws_config_metadata = result.stdout.decode("utf-8").splitlines()
    return get_sdk_versions(desired_versions_count, aws_config_metadata)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", dest="self_test", action="store_true",
                        help="Run unit tests for this script")
    parser.add_argument("--sdk-versions", dest="versions", default=3, type=int,
                        help="Number of versions to include in the matrix")
    parser.add_argument("rust_versions", type=str, nargs=argparse.ONE_OR_MORE)

    args = parser.parse_args()
    if args.self_test:
        sys.argv.pop()
        unittest.main()
    else:
        versions = acquire_sdk_versions(args.versions)
        output = {
            "sdk_version": versions,
            "rust_version": args.rust_versions
        }
        print(json.dumps(output))


class SelfTest(unittest.TestCase):
    def test_get_sdk_versions(self):
        lines = [
            json.dumps({"vers": "0.0.22-alpha", "yanked": False}),
            json.dumps({"vers": "0.0.23-alpha", "yanked": False}),
            json.dumps({"vers": "0.0.24-alpha", "yanked": True}),
            json.dumps({"vers": "0.0.25-alpha", "yanked": False}),
            json.dumps({"vers": "0.0.26-alpha", "yanked": False}),
            json.dumps({"vers": "0.2.0", "yanked": False}),
            json.dumps({"vers": "0.3.0", "yanked": False}),
        ]
        self.assertEqual(["0.0.26-alpha", "0.2.0", "0.3.0"],
                         get_sdk_versions(3, lines))
        self.assertEqual(["0.0.22-alpha", "0.0.23-alpha", "0.0.25-alpha", "0.0.26-alpha", "0.2.0", "0.3.0"],
                         get_sdk_versions(7, lines))


if __name__ == "__main__":
    main()
