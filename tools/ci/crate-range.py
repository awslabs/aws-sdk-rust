#!/usr/bin/env -S python3 -u
#
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0.
#

# This script is used by CI to split the SDK crates up into batches that can be
# compiled and tested in parallel by multiple machines. It has two sub-commands
# to accomplish this:
#
# generate-matrix
#
#   This sub-command counts the total number of crates in the SDK (including examples),
#   and creates a set of batches based on that size in a format that GitHub Actions
#   can consume as a matrix strategy.
#
# run
#
#   This sub-command will run the given command on the given crate range.
#   It deterministically distributes the SDK crates (including examples) fairly across
#   the batch range by counting lines of code in attempt to keep batches with roughly
#   the same workloads.
#
#   For example, if there are crates:
#
#     Crate Name    Lines of Code
#     ----------    -------------
#         A             1000
#         B             1000
#         C             1000
#         D              500
#         E              500
#         F              500
#
#   And if there are 3 batches, this will end up getting sorted as: A, D, B, E, C, F.
#   Then the range operation will be performed on it. So if the range is "-s 2 -e 4",
#   then the command would be run on B and E.
#
# self-test
#
#   Runs unit tests for this script.


import argparse
import json
import math
import os
import shlex
import subprocess
import sys
import unittest

COLOR_YELLOW = "\033[93m"
COLOR_RESET = "\033[0m"


class Crate:
    def __init__(self, path, loc=None):
        self.path = path
        self.loc = loc

    # Counts the lines of code on this crate and caches it
    def count_lines(self):
        if self.loc is None:
            self.loc = int(get_cmd_output(
                f"find {self.path} -name '*.rs' -exec wc -l {{}} \\; | \
                cut -d' ' -f1 | \
                paste -sd+ | \
                bc",
                shell=True
            ))

    def __repr__(self):
        return "Crate" + str((self.loc, self.path))

    def __eq__(self, other):
        return isinstance(other, Crate) and self.path == other.path


# Given the total number of jobs and batch count, this calculates the actual batch ranges
def calculate_batches(total_jobs, batch_count):
    batch_size = math.ceil(total_jobs / batch_count)
    return list(map(lambda i: (i, min(total_jobs, i + batch_size)), range(0, total_jobs, batch_size)))


# Splits a list into N lists
def split_list(list, n):
    result = []
    for batch in calculate_batches(len(list), n):
        result.append(list[batch[0]:batch[1]])
    return result


# Optimistically sorts the crate list in such a way that the total work per batch is more balanced.
# Accomplishes this by sorting by most lines of code to least lines of code, and then striping that
# list across batches. Finally, it combines those batches back into one list.
#
# IMPORTANT: This must be deterministic and return the same list for the same files every time
def organize_crate_list(batch_count, crates):
    crates = sorted(crates, key=lambda c: c.loc, reverse=True)
    batches = list(map(lambda _: [], [0] * batch_count))  # Can't do `[] * n` for some reason
    for index, crate in enumerate(crates):
        batches[index % len(batches)].append(crate)

    result = []
    for batch in batches:
        result.extend(batch)

    return result


# Lists all SDK crates including examples
def list_crates(repository_root, path):
    to_examine = []
    to_examine.extend(list(map(lambda p: f"{repository_root}/{path}/{p}", os.listdir(f"{repository_root}/{path}"))))

    crates = []
    for path in to_examine:
        if os.path.isfile(f"{path}/Cargo.toml"):
            crates.append(Crate(path))

    return crates


# Entry point for the `generate-matrix` sub-command
def subcommand_generate_matrix(repository_root, batch_count, folder, rust_versions):
    crates = list_crates(repository_root, folder)
    batches = calculate_batches(len(crates), batch_count)

    output = {
        "crate_range": list(map(lambda b: f"-b {batch_count} -s {b[0]} -e {b[1]}", batches)),
        "rust_version": rust_versions
    }
    print(json.dumps(output))
    return 0


# Entry point for the `run` sub-command
def subcommand_run(repository_root, batch_count, start_inclusive, end_exclusive, folder, command):
    print(f"{COLOR_YELLOW}Determining crate list...{COLOR_RESET}")
    crates = list_crates(repository_root, folder)

    if end_exclusive <= start_inclusive or end_exclusive < 0 or start_inclusive < 0:
        print("Invalid range")
        return 1
    if start_inclusive >= len(crates):
        print("Range start is invalid")
        return 1
    if end_exclusive > len(crates):
        print("Range end is invalid")
        return 1

    for crate in crates:
        crate.count_lines()
    crates = organize_crate_list(batch_count, crates)
    crates = crates[start_inclusive:end_exclusive]

    print(f"{COLOR_YELLOW}Crates to run against:{COLOR_RESET}")
    for crate in crates:
        print(f"{COLOR_YELLOW}  {crate.loc}\t{crate.path}{COLOR_RESET}")

    completed = 0
    for crate in crates:
        print(f"{COLOR_YELLOW}Current crate: {crate.path}, loc: {crate.loc}, completed: {completed}, "
              f"remaining: {len(crates) - completed}{COLOR_RESET}")
        os.chdir(crate.path)
        subprocess.run(command, check=True)
        completed += 1

    return 0


def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(required=True, dest="subcommand")

    subparsers.add_parser("self-test", help="Run unit tests for this script")

    subparser_generate_matrix = subparsers.add_parser("generate-matrix", help="Generate a test matrix")
    subparser_generate_matrix.add_argument("rust_versions", type=str, nargs=argparse.ONE_OR_MORE)
    subparser_generate_matrix.add_argument("-b", type=int, dest="batches", required=True, help="Number of batches")
    subparser_generate_matrix.add_argument("--folder", required=True, type=str, choices=["sdk", "examples"],
                                           help="Name of the folder containing the crates you want to generate a "
                                                "matrix for") 

    subparser_run = subparsers.add_parser("run", help="Run command on crate range")
    subparser_run.add_argument("-b", required=True, type=int, dest="batches", help="Number of batches")
    subparser_run.add_argument("-s", required=True, type=int, dest="start_inclusive", help="Range start inclusive")
    subparser_run.add_argument("-e", required=True, type=int, dest="end_exclusive", help="Range end exclusive")
    subparser_run.add_argument("--folder", required=True, type=str, choices=["sdk", "examples"],
                               help="Name of the folder containing the crates you want to run a command on")
    subparser_run.add_argument("cmd", type=str, nargs=argparse.ONE_OR_MORE)

    args = parser.parse_args()

    repository_root = get_cmd_output("git rev-parse --show-toplevel")
    if args.subcommand == "self-test":
        sys.argv.pop()
        unittest.main()
        return 0
    elif args.subcommand == "generate-matrix":
        return subcommand_generate_matrix(repository_root, args.batches, args.folder, args.rust_versions)
    elif args.subcommand == "run":
        return subcommand_run(repository_root, args.batches, args.start_inclusive, args.end_exclusive, args.folder, args.cmd)


def get_cmd_output(command, shell=False):
    if not shell:
        command = shlex.split(command)
    result = subprocess.run(command, shell=shell, capture_output=True, check=True)
    return result.stdout.decode("utf-8").strip()


class SelfTest(unittest.TestCase):
    def test_split_list(self):
        self.assertEqual([[1, 2], [3, 4], [5, 6]], split_list([1, 2, 3, 4, 5, 6], 3))
        self.assertEqual([[1, 2], [3, 4], [5]], split_list([1, 2, 3, 4, 5], 3))

    def test_calculate_batches(self):
        self.assertEqual([(0, 2), (2, 4), (4, 5)], calculate_batches(5, 3))
        self.assertEqual([(0, 2), (2, 4), (4, 6)], calculate_batches(6, 3))
        self.assertEqual([(0, 56), (56, 111)], calculate_batches(111, 2))
        self.assertEqual([(0, 111)], calculate_batches(111, 1))
        self.assertEqual([(0, 1), (1, 2)], calculate_batches(2, 10))

    def test_organize_crate_list(self):
        self.assertEqual(
            [
                # batch 1
                Crate("A", 3000),
                Crate("C", 1000),
                Crate("E", 500),
                Crate("G", 200),
                Crate("I", 50),

                # batch 2
                Crate("B", 2000),
                Crate("D", 1000),
                Crate("F", 300),
                Crate("H", 100),
            ],
            organize_crate_list(2, [
                Crate("A", 3000),
                Crate("B", 2000),
                Crate("C", 1000),
                Crate("D", 1000),
                Crate("E", 500),
                Crate("F", 300),
                Crate("G", 200),
                Crate("H", 100),
                Crate("I", 50),
            ])
        )
        self.assertEqual(
            [
                # batch 1
                Crate("A", 3000),
                Crate("D", 1000),
                Crate("G", 200),

                # batch 2
                Crate("B", 2000),
                Crate("E", 500),
                Crate("H", 100),

                # batch 3
                Crate("C", 1000),
                Crate("F", 300),
                Crate("I", 50),
            ],
            organize_crate_list(3, [
                Crate("A", 3000),
                Crate("B", 2000),
                Crate("C", 1000),
                Crate("D", 1000),
                Crate("E", 500),
                Crate("F", 300),
                Crate("G", 200),
                Crate("H", 100),
                Crate("I", 50),
            ])
        )


if __name__ == "__main__":
    sys.exit(main())
