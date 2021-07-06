/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_ebs::model::ChecksumAlgorithm;
use aws_sdk_ebs::ByteStream;
use aws_sdk_ec2::model::Filter;
use sha2::Digest;

/// EBS only supports one fixed size of block
const EBS_BLOCK_SIZE: usize = 524288;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_ebs::Error> {
    tracing_subscriber::fmt::init();
    let client = aws_sdk_ebs::Client::from_env();
    let snapshot = client
        .start_snapshot()
        .description("new_snapshot")
        .encrypted(false)
        .volume_size(1)
        .send()
        .await?;
    println!("snapshot started: {:?}", snapshot);
    let snapshot_id = snapshot.snapshot_id.unwrap();
    let mut blocks = vec![];
    // append a block of all 1s
    let mut block: Vec<u8> = Vec::new();
    block.resize(EBS_BLOCK_SIZE, 1);
    blocks.push(block);

    // append a block of all 0s
    let mut block: Vec<u8> = Vec::new();
    block.resize(EBS_BLOCK_SIZE, 0);
    blocks.push(block);

    for (idx, block) in blocks.into_iter().enumerate() {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&block);
        let checksum = hasher.finalize();
        let checksum = base64::encode(&checksum[..]);

        client
            .put_snapshot_block()
            .snapshot_id(&snapshot_id)
            .block_index(idx as i32)
            .block_data(ByteStream::from(block))
            .checksum(checksum)
            .checksum_algorithm(ChecksumAlgorithm::ChecksumAlgorithmSha256)
            .data_length(EBS_BLOCK_SIZE as i32)
            .send()
            .await?;
    }
    let rsp = client
        .complete_snapshot()
        .changed_blocks_count(2)
        .snapshot_id(&snapshot_id)
        .send()
        .await?;
    println!("snapshot complete: {:#?}", rsp);

    // NOTE: you need to wait for `status != pending`
    let ec2_client = aws_sdk_ec2::Client::from_env();
    let snapshots = ec2_client
        .describe_snapshots()
        .filters(
            Filter::builder()
                .name("snapshot-id")
                .values(&snapshot_id)
                .build(),
        )
        .send()
        .await;
    println!("snapshot status: {:#?}", snapshots);

    Ok(())
}
