# aws-sdk-support

The _Amazon Web Services Support API Reference_ is intended for programmers who need detailed information about the Amazon Web Services Support operations and data types. You can use the API to manage your support cases programmatically. The Amazon Web Services Support API uses HTTP methods that return results in JSON format.

You can also use the Amazon Web Services Support API to access features for [Trusted Advisor](http://aws.amazon.com/premiumsupport/trustedadvisor/). You can return a list of checks and their descriptions, get check results, specify checks to refresh, and get the refresh status of checks.

You can manage your support cases with the following Amazon Web Services Support API operations:
  - The CreateCase, DescribeCases, DescribeAttachment, and ResolveCase operations create Amazon Web Services Support cases, retrieve information about cases, and resolve cases.
  - The DescribeCommunications, AddCommunicationToCase, and AddAttachmentsToSet operations retrieve and add communications and attachments to Amazon Web Services Support cases.
  - The DescribeServices and DescribeSeverityLevels operations return Amazon Web Service names, service codes, service categories, and problem severity levels. You use these values when you call the CreateCase operation.

You can also use the Amazon Web Services Support API to call the Trusted Advisor operations. For more information, see [Trusted Advisor](https://docs.aws.amazon.com/) in the _Amazon Web Services Support User Guide_.

For authentication of requests, Amazon Web Services Support uses [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html).

For more information about this service and the endpoints to use, see [About the Amazon Web Services Support API](https://docs.aws.amazon.com/awssupport/latest/user/about-support-api.html) in the _Amazon Web Services Support User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-support` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-support = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_support as support;

#[::tokio::main]
async fn main() -> Result<(), support::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_support::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-support/latest/aws_sdk_support/client/struct.Client.html)
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

