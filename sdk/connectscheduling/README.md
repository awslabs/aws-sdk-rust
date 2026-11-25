# aws-sdk-connectscheduling

  - [Scheduling actions](https://docs.aws.amazon.com/connect/latest/APIReference/API_Operations_Amazon_Connect_Scheduling.html)
  - [Scheduling data types](https://docs.aws.amazon.com/connect/latest/APIReference/API_Types_Amazon_Connect_Scheduling.html)

Use the Amazon Connect Scheduling API to work programmatically with Connect Customer forecasting, capacity planning, and scheduling, and to integrate these capabilities with your own systems. In the Connect Customer admin website, you use them to forecast contact volume, plan capacity, and create and manage agent schedules.

For more information, see [Forecasting & agent scheduling](https://docs.aws.amazon.com/connect/latest/adminguide/forecasting-capacity-planning-scheduling.html) in the _Connect Customer Administrator Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-connectscheduling` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-connectscheduling = "1.2.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_connectscheduling as connectscheduling;

#[::tokio::main]
async fn main() -> Result<(), connectscheduling::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_connectscheduling::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-connectscheduling/latest/aws_sdk_connectscheduling/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

