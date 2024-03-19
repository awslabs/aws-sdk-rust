# aws-sdk-signer

AWS Signer is a fully managed code-signing service to help you ensure the trust and integrity of your code.

Signer supports the following applications:

With code signing for AWS Lambda, you can sign [AWS Lambda](http://docs.aws.amazon.com/lambda/latest/dg/) deployment packages. Integrated support is provided for [Amazon S3](http://docs.aws.amazon.com/AmazonS3/latest/gsg/), [Amazon CloudWatch](http://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/), and [AWS CloudTrail](http://docs.aws.amazon.com/awscloudtrail/latest/userguide/). In order to sign code, you create a signing profile and then use Signer to sign Lambda zip files in S3.

With code signing for IoT, you can sign code for any IoT device that is supported by AWS. IoT code signing is available for [Amazon FreeRTOS](http://docs.aws.amazon.com/freertos/latest/userguide/) and [AWS IoT Device Management](http://docs.aws.amazon.com/iot/latest/developerguide/), and is integrated with [AWS Certificate Manager (ACM)](http://docs.aws.amazon.com/acm/latest/userguide/). In order to sign code, you import a third-party code-signing certificate using ACM, and use that to sign updates in Amazon FreeRTOS and AWS IoT Device Management.

With Signer and the Notation CLI from the [Notary  Project](https://notaryproject.dev/), you can sign container images stored in a container registry such as Amazon Elastic Container Registry (ECR). The signatures are stored in the registry alongside the images, where they are available for verifying image authenticity and integrity.

For more information about Signer, see the [AWS Signer Developer Guide](https://docs.aws.amazon.com/signer/latest/developerguide/Welcome.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-signer` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-signer = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_signer as signer;

#[::tokio::main]
async fn main() -> Result<(), signer::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_signer::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-signer/latest/aws_sdk_signer/client/struct.Client.html)
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

