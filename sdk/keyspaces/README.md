# aws-sdk-keyspaces

Amazon Keyspaces (for Apache Cassandra) is a scalable, highly available, and managed Apache Cassandra-compatible database service. Amazon Keyspaces makes it easy to migrate, run, and scale Cassandra workloads in the Amazon Web Services Cloud. With just a few clicks on the Amazon Web Services Management Console or a few lines of code, you can create keyspaces and tables in Amazon Keyspaces, without deploying any infrastructure or installing software.

In addition to supporting Cassandra Query Language (CQL) requests via open-source Cassandra drivers, Amazon Keyspaces supports data definition language (DDL) operations to manage keyspaces and tables using the Amazon Web Services SDK and CLI, as well as infrastructure as code (IaC) services and tools such as CloudFormation and Terraform. This API reference describes the supported DDL operations in detail.

For the list of all supported CQL APIs, see [Supported Cassandra APIs, operations, and data types in Amazon Keyspaces](https://docs.aws.amazon.com/keyspaces/latest/devguide/cassandra-apis.html) in the _Amazon Keyspaces Developer Guide_.

To learn how Amazon Keyspaces API actions are recorded with CloudTrail, see [Amazon Keyspaces information in CloudTrail](https://docs.aws.amazon.com/keyspaces/latest/devguide/logging-using-cloudtrail.html#service-name-info-in-cloudtrail) in the _Amazon Keyspaces Developer Guide_.

For more information about Amazon Web Services APIs, for example how to implement retry logic or how to sign Amazon Web Services API requests, see [Amazon Web Services APIs](https://docs.aws.amazon.com/general/latest/gr/aws-apis.html) in the _General Reference_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-keyspaces` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-keyspaces = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_keyspaces as keyspaces;

#[::tokio::main]
async fn main() -> Result<(), keyspaces::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_keyspaces::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-keyspaces/latest/aws_sdk_keyspaces/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

