# aws-sdk-dataexchange

AWS Data Exchange is a service that makes it easy for AWS customers to exchange data in the cloud. You can use the AWS Data Exchange APIs to create, update, manage, and access file-based data set in the AWS Cloud.

As a subscriber, you can view and access the data sets that you have an entitlement to through a subscription. You can use the APIs to download or copy your entitled data sets to Amazon Simple Storage Service (Amazon S3) for use across a variety of AWS analytics and machine learning services.

As a provider, you can create and manage your data sets that you would like to publish to a product. Being able to package and provide your data sets into products requires a few steps to determine eligibility. For more information, visit the _AWS Data Exchange User Guide_.

A data set is a collection of data that can be changed or updated over time. Data sets can be updated using revisions, which represent a new version or incremental change to a data set. A revision contains one or more assets. An asset in AWS Data Exchange is a piece of data that can be stored as an Amazon S3 object, Redshift datashare, API Gateway API, AWS Lake Formation data permission, or Amazon S3 data access. The asset can be a structured data file, an image file, or some other data file. Jobs are asynchronous import or export operations used to create or copy assets.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-dataexchange` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-dataexchange = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_dataexchange as dataexchange;

#[::tokio::main]
async fn main() -> Result<(), dataexchange::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dataexchange::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-dataexchange/latest/aws_sdk_dataexchange/client/struct.Client.html)
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

