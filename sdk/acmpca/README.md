# aws-sdk-acmpca

This is the _Amazon Web Services Private Certificate Authority API Reference_. It provides descriptions, syntax, and usage examples for each of the actions and data types involved in creating and managing a private certificate authority (CA) for your organization.

The documentation for each action shows the API request parameters and the JSON response. Alternatively, you can use one of the Amazon Web Services SDKs to access an API that is tailored to the programming language or platform that you prefer. For more information, see [Amazon Web Services SDKs](https://aws.amazon.com/tools/#SDKs).

Each Amazon Web Services Private CA API operation has a quota that determines the number of times the operation can be called per second. Amazon Web Services Private CA throttles API requests at different rates depending on the operation. Throttling means that Amazon Web Services Private CA rejects an otherwise valid request because the request exceeds the operation's quota for the number of requests per second. When a request is throttled, Amazon Web Services Private CA returns a [ThrottlingException](https://docs.aws.amazon.com/privateca/latest/APIReference/CommonErrors.html) error. Amazon Web Services Private CA does not guarantee a minimum request rate for APIs.

To see an up-to-date list of your Amazon Web Services Private CA quotas, or to request a quota increase, log into your Amazon Web Services account and visit the [Service Quotas](https://console.aws.amazon.com/servicequotas/) console.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-acmpca` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-acmpca = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_acmpca as acmpca;

#[::tokio::main]
async fn main() -> Result<(), acmpca::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_acmpca::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-acmpca/latest/aws_sdk_acmpca/client/struct.Client.html)
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

