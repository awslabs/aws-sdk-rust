# aws-sdk-appflow

Welcome to the Amazon AppFlow API reference. This guide is for developers who need detailed information about the Amazon AppFlow API operations, data types, and errors.

Amazon AppFlow is a fully managed integration service that enables you to securely transfer data between software as a service (SaaS) applications like Salesforce, Marketo, Slack, and ServiceNow, and Amazon Web Services like Amazon S3 and Amazon Redshift.

Use the following links to get started on the Amazon AppFlow API:
  - [Actions](https://docs.aws.amazon.com/appflow/1.0/APIReference/API_Operations.html): An alphabetical list of all Amazon AppFlow API operations.
  - [Data types](https://docs.aws.amazon.com/appflow/1.0/APIReference/API_Types.html): An alphabetical list of all Amazon AppFlow data types.
  - [Common parameters](https://docs.aws.amazon.com/appflow/1.0/APIReference/CommonParameters.html): Parameters that all Query operations can use.
  - [Common errors](https://docs.aws.amazon.com/appflow/1.0/APIReference/CommonErrors.html): Client and server errors that all operations can return.

If you're new to Amazon AppFlow, we recommend that you review the [Amazon AppFlow User Guide](https://docs.aws.amazon.com/appflow/latest/userguide/what-is-appflow.html).

Amazon AppFlow API users can use vendor-specific mechanisms for OAuth, and include applicable OAuth attributes (such as auth-code and redirecturi) with the connector-specific ConnectorProfileProperties when creating a new connector profile using Amazon AppFlow API operations. For example, Salesforce users can refer to the [_Authorize Apps with OAuth_](https://help.salesforce.com/articleView?id=remoteaccess_authenticate.htm) documentation.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appflow` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-appflow = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_appflow as appflow;

#[::tokio::main]
async fn main() -> Result<(), appflow::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_appflow::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-appflow/latest/aws_sdk_appflow/client/struct.Client.html)
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

