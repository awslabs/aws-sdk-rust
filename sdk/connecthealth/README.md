# aws-sdk-connecthealth

Amazon Connect Health is an AI-powered healthcare service built on Amazon Connect. It provides pre-built agents that automate patient engagement workflows and support clinical documentation at the point of care.

You can use the Amazon Connect Health API to programmatically manage domains, configure patient engagement agents, run patient insights jobs, and stream ambient documentation sessions. This API reference describes the available API operations and data types for Amazon Connect Health.

We recommend that you use the AWS SDKs to make programmatic API calls to Amazon Connect Health.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-connecthealth` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-connecthealth = "1.9.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_connecthealth as connecthealth;

#[::tokio::main]
async fn main() -> Result<(), connecthealth::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_connecthealth::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-connecthealth/latest/aws_sdk_connecthealth/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

