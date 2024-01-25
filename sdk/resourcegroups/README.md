# aws-sdk-resourcegroups

Resource Groups lets you organize Amazon Web Services resources such as Amazon Elastic Compute Cloud instances, Amazon Relational Database Service databases, and Amazon Simple Storage Service buckets into groups using criteria that you define as tags. A resource group is a collection of resources that match the resource types specified in a query, and share one or more tags or portions of tags. You can create a group of resources based on their roles in your cloud infrastructure, lifecycle stages, regions, application layers, or virtually any criteria. Resource Groups enable you to automate management tasks, such as those in Amazon Web Services Systems Manager Automation documents, on tag-related resources in Amazon Web Services Systems Manager. Groups of tagged resources also let you quickly view a custom console in Amazon Web Services Systems Manager that shows Config compliance and other monitoring data about member resources.

To create a resource group, build a resource query, and specify tags that identify the criteria that members of the group have in common. Tags are key-value pairs.

For more information about Resource Groups, see the [Resource Groups User Guide](https://docs.aws.amazon.com/ARG/latest/userguide/welcome.html).

Resource Groups uses a REST-compliant API that you can use to perform the following types of operations.
  - Create, Read, Update, and Delete (CRUD) operations on resource groups and resource query entities
  - Applying, editing, and removing tags from resource groups
  - Resolving resource group member ARNs so they can be returned as search results
  - Getting data about resources that are members of a group
  - Searching Amazon Web Services resources based on a resource query

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-resourcegroups` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-resourcegroups = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_resourcegroups as resourcegroups;

#[::tokio::main]
async fn main() -> Result<(), resourcegroups::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_resourcegroups::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-resourcegroups/latest/aws_sdk_resourcegroups/client/struct.Client.html)
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

