/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fs::File;
use std::io::Read;

mod common;

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    println!("cwd: {:?}", current_dir);

    let corpus_path = "corpus/json_deserialize";
    for entry in std::fs::read_dir(corpus_path).unwrap() {
        let path = entry.unwrap().path();
        println!("Running {:?}", path);

        let mut data = Vec::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut data).unwrap();

        common::run_data(&data);
    }
}
