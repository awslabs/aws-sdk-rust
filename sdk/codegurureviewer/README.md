# aws-sdk-codegurureviewer

This section provides documentation for the Amazon CodeGuru Reviewer API operations. CodeGuru Reviewer is a service that uses program analysis and machine learning to detect potential defects that are difficult for developers to find and recommends fixes in your Java and Python code.

By proactively detecting and providing recommendations for addressing code defects and implementing best practices, CodeGuru Reviewer improves the overall quality and maintainability of your code base during the code review stage. For more information about CodeGuru Reviewer, see the _ [Amazon CodeGuru Reviewer User Guide](https://docs.aws.amazon.com/codeguru/latest/reviewer-ug/welcome.html)._

To improve the security of your CodeGuru Reviewer API calls, you can establish a private connection between your VPC and CodeGuru Reviewer by creating an _interface VPC endpoint_. For more information, see [CodeGuru Reviewer and interface VPC endpoints (Amazon Web Services PrivateLink)](https://docs.aws.amazon.com/codeguru/latest/reviewer-ug/vpc-interface-endpoints.html) in the _Amazon CodeGuru Reviewer User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codegurureviewer` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-codegurureviewer = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_codegurureviewer as codegurureviewer;

#[::tokio::main]
async fn main() -> Result<(), codegurureviewer::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_codegurureviewer::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-codegurureviewer/latest/aws_sdk_codegurureviewer/client/struct.Client.html)
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

