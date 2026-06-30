# aws-sdk-cloudwatch

Amazon CloudWatch enables you to publish, monitor, and manage various metrics, as well as configure alarm actions based on data from metrics. This guide provides detailed information about CloudWatch actions, data types, parameters, and errors. For more information about CloudWatch features, see [Amazon CloudWatch](https://aws.amazon.com/cloudwatch) and the _Amazon CloudWatch User Guide_.

For information about the metrics that other Amazon Web Services products send to CloudWatch, see the [Amazon CloudWatch Metrics and Dimensions Reference](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/aws-services-cloudwatch-metrics.html) in the _Amazon CloudWatch User Guide_.

Use the following links to get started using the CloudWatch Query API:

: An alphabetical list of all CloudWatch actions.

: An alphabetical list of all CloudWatch data types.

CommonParameters: Parameters that all Query actions can use.

CommonErrors: Client and server errors that all actions can return.

[Regions and Endpoints](https://docs.aws.amazon.com/general/latest/gr/rande.html#cw_region): Supported regions and endpoints for all Amazon Web Services products.

Alternatively, you can use one of the [Amazon Web Services SDKs](https://aws.amazon.com/tools/#sdk) to access CloudWatch using an API tailored to your programming language or platform.

Developers in the Amazon Web Services developer community also provide their own libraries, which you can find at the following Amazon Web Services developer centers:

[Java Developer Center](http://aws.amazon.com/java/)

[JavaScript Developer Center](http://aws.amazon.com/javascript/)

[Amazon Web Services Mobile Services](http://aws.amazon.com/mobile/)

[PHP Developer Center](http://aws.amazon.com/php/)

[Python Developer Center](http://aws.amazon.com/python/)

[Ruby Developer Center](http://aws.amazon.com/ruby/)

[Windows and .NET Developer Center](http://aws.amazon.com/net/)

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-cloudwatch` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-cloudwatch = "1.117.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_cloudwatch as cloudwatch;

#[::tokio::main]
async fn main() -> Result<(), cloudwatch::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_cloudwatch::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-cloudwatch/latest/aws_sdk_cloudwatch/client/struct.Client.html)
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

