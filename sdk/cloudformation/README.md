# aws-sdk-cloudformation

CloudFormation allows you to create and manage Amazon Web Services infrastructure deployments predictably and repeatedly. You can use CloudFormation to leverage Amazon Web Services products, such as Amazon Elastic Compute Cloud, Amazon Elastic Block Store, Amazon Simple Notification Service, Elastic Load Balancing, and Auto Scaling to build highly reliable, highly scalable, cost-effective applications without creating or configuring the underlying Amazon Web Services infrastructure.

With CloudFormation, you declare all your resources and dependencies in a template file. The template defines a collection of resources as a single unit called a stack. CloudFormation creates and deletes all member resources of the stack together and manages all dependencies between the resources for you.

For more information about CloudFormation, see the [CloudFormation product page](http://aws.amazon.com/cloudformation/).

CloudFormation makes use of other Amazon Web Services products. If you need additional technical information about a specific Amazon Web Services product, you can find the product's technical documentation at [docs.aws.amazon.com](https://docs.aws.amazon.com/).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-cloudformation` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-cloudformation = "1.13.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_cloudformation as cloudformation;

#[::tokio::main]
async fn main() -> Result<(), cloudformation::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_cloudformation::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-cloudformation/latest/aws_sdk_cloudformation/client/struct.Client.html)
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

