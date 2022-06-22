# aws-sdk-signer

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

AWS Signer is a fully managed code signing service to help you ensure the trust and integrity of your code.

AWS Signer supports the following applications:

With _code signing for AWS Lambda_, you can sign AWS Lambda deployment packages. Integrated support is provided for Amazon S3, Amazon CloudWatch, and AWS CloudTrail. In order to sign code, you create a signing profile and then use Signer to sign Lambda zip files in S3.

With _code signing for IoT_, you can sign code for any IoT device that is supported by AWS. IoT code signing is available for [Amazon FreeRTOS](http://docs.aws.amazon.com/freertos/latest/userguide/) and [AWS IoT Device Management](http://docs.aws.amazon.com/iot/latest/developerguide/), and is integrated with [AWS Certificate Manager (ACM)](http://docs.aws.amazon.com/acm/latest/userguide/). In order to sign code, you import a third-party code signing certificate using ACM, and use that to sign updates in Amazon FreeRTOS and AWS IoT Device Management.

For more information about AWS Signer, see the [AWS Signer Developer Guide](http://docs.aws.amazon.com/signer/latest/developerguide/Welcome.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-signer` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-signer = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

