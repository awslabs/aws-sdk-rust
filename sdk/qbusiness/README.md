# aws-sdk-qbusiness

This is the _Amazon Q Business_ API Reference. Amazon Q Business is a fully managed, generative-AI powered enterprise chat assistant that you can deploy within your organization. Amazon Q Business enhances employee productivity by supporting key tasks such as question-answering, knowledge discovery, writing email messages, summarizing text, drafting document outlines, and brainstorming ideas. Users ask questions of Amazon Q Business and get answers that are presented in a conversational manner. For an introduction to the service, see the [_Amazon Q Business User Guide_](https://docs.aws.amazon.com/amazonq/latest/business-use-dg/what-is.html).

For an overview of the Amazon Q Business APIs, see [Overview of Amazon Q Business API operations](https://docs.aws.amazon.com/amazonq/latest/business-use-dg/api-ref.html#api-overview).

For information about the IAM access control permissions you need to use this API, see [IAM roles for Amazon Q Business](https://docs.aws.amazon.com/amazonq/latest/business-use-dg/iam-roles.html) in the _Amazon Q Business User Guide_.

The following resources provide additional information about using the Amazon Q Business API:
  - _ [Setting up for Amazon Q Business](https://docs.aws.amazon.com/amazonq/latest/business-use-dg/setting-up.html) _
  - _ [Amazon Q Business CLI Reference](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/qbusiness/index.html) _
  - _ [Amazon Web Services General Reference](https://docs.aws.amazon.com/general/latest/gr/amazonq.html) _

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-qbusiness` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-qbusiness = "1.94.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_qbusiness as qbusiness;

#[::tokio::main]
async fn main() -> Result<(), qbusiness::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_qbusiness::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-qbusiness/latest/aws_sdk_qbusiness/client/struct.Client.html)
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

