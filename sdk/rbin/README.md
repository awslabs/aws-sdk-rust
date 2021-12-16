# aws-sdk-rbin

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the _Recycle Bin API Reference_. This documentation provides descriptions and syntax for each of the actions and data types in Recycle Bin.

Recycle Bin is a snapshot recovery feature that enables you to restore accidentally deleted snapshots. When using Recycle Bin, if your snapshots are deleted, they are retained in the Recycle Bin for a time period that you specify.

You can restore a snapshot from the Recycle Bin at any time before its retention period expires. After you restore a snapshot from the Recycle Bin, the snapshot is removed from the Recycle Bin, and you can then use it in the same way you use any other snapshot in your account. If the retention period expires and the snapshot is not restored, the snapshot is permanently deleted from the Recycle Bin and is no longer available for recovery. For more information about Recycle Bin, see [Recycle Bin](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/snapshot-recycle-bin.html) in the _Amazon EC2 User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-rbin` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.3.0"
aws-sdk-rbin = "0.3.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Guide](https://github.com/awslabs/aws-sdk-rust/blob/main/Guide.md). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

