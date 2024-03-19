# aws-sdk-resourceexplorer2

Amazon Web Services Resource Explorer is a resource search and discovery service. By using Resource Explorer, you can explore your resources using an internet search engine-like experience. Examples of resources include Amazon Relational Database Service (Amazon RDS) instances, Amazon Simple Storage Service (Amazon S3) buckets, or Amazon DynamoDB tables. You can search for your resources using resource metadata like names, tags, and IDs. Resource Explorer can search across all of the Amazon Web Services Regions in your account in which you turn the service on, to simplify your cross-Region workloads.

Resource Explorer scans the resources in each of the Amazon Web Services Regions in your Amazon Web Services account in which you turn on Resource Explorer. Resource Explorer [creates and maintains an index](https://docs.aws.amazon.com/resource-explorer/latest/userguide/getting-started-terms-and-concepts.html#term-index) in each Region, with the details of that Region's resources.

You can [search across all of the indexed Regions in your account](https://docs.aws.amazon.com/resource-explorer/latest/userguide/manage-aggregator-region.html) by designating one of your Amazon Web Services Regions to contain the aggregator index for the account. When you [promote a local index in a Region to become the aggregator index for the account](https://docs.aws.amazon.com/resource-explorer/latest/userguide/manage-aggregator-region-turn-on.html), Resource Explorer automatically replicates the index information from all local indexes in the other Regions to the aggregator index. Therefore, the Region with the aggregator index has a copy of all resource information for all Regions in the account where you turned on Resource Explorer. As a result, views in the aggregator index Region include resources from all of the indexed Regions in your account.

For more information about Amazon Web Services Resource Explorer, including how to enable and configure the service, see the [Amazon Web Services Resource Explorer User Guide](https://docs.aws.amazon.com/resource-explorer/latest/userguide/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-resourceexplorer2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-resourceexplorer2 = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_resourceexplorer2 as resourceexplorer2;

#[::tokio::main]
async fn main() -> Result<(), resourceexplorer2::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_resourceexplorer2::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-resourceexplorer2/latest/aws_sdk_resourceexplorer2/client/struct.Client.html)
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

