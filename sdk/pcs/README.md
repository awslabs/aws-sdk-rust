# aws-sdk-pcs

Amazon Web Services Parallel Computing Service (Amazon Web Services PCS) is a managed service that makes it easier for you to run and scale your high performance computing (HPC) workloads, and build scientific and engineering models on Amazon Web Services using Slurm. For more information, see the [Amazon Web Services Parallel Computing Service User Guide](https://docs.aws.amazon.com/pcs/latest/userguide).

This reference describes the actions and data types of the service management API. You can use the Amazon Web Services SDKs to call the API actions in software, or use the Command Line Interface (CLI) to call the API actions manually. These API actions manage the service through an Amazon Web Services account.

The API actions operate on Amazon Web Services PCS resources. A _resource_ is an entity in Amazon Web Services that you can work with. Amazon Web Services services create resources when you use the features of the service. Examples of Amazon Web Services PCS resources include clusters, compute node groups, and queues. For more information about resources in Amazon Web Services, see [Resource](https://docs.aws.amazon.com/resource-explorer/latest/userguide/getting-started-terms-and-concepts.html#term-resource) in the _Resource Explorer User Guide_.

An Amazon Web Services PCS _compute node_ is an Amazon EC2 instance. You don't launch compute nodes directly. Amazon Web Services PCS uses configuration information that you provide to launch compute nodes in your Amazon Web Services account. You receive billing charges for your running compute nodes. Amazon Web Services PCS automatically terminates your compute nodes when you delete the Amazon Web Services PCS resources related to those compute nodes.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-pcs` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-pcs = "1.43.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_pcs as pcs;

#[::tokio::main]
async fn main() -> Result<(), pcs::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_pcs::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-pcs/latest/aws_sdk_pcs/client/struct.Client.html)
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

