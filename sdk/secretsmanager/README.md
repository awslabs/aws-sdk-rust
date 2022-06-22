# aws-sdk-secretsmanager

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Secrets Manager provides a service to enable you to store, manage, and retrieve, secrets.

This guide provides descriptions of the Secrets Manager API. For more information about using this service, see the [Amazon Web Services Secrets Manager User Guide](https://docs.aws.amazon.com/secretsmanager/latest/userguide/introduction.html).

__API Version__

This version of the Secrets Manager API Reference documents the Secrets Manager API version 2017-10-17.

__Support and Feedback for Amazon Web Services Secrets Manager__

We welcome your feedback. Send your comments to [awssecretsmanager-feedback@amazon.com](mailto:awssecretsmanager-feedback@amazon.com), or post your feedback and questions in the [Amazon Web Services Secrets Manager Discussion Forum](http://forums.aws.amazon.com/forum.jspa?forumID=296). For more information about the Amazon Web Services Discussion Forums, see [Forums Help](http://forums.aws.amazon.com/help.jspa).

__Logging API Requests__

Amazon Web Services Secrets Manager supports Amazon Web Services CloudTrail, a service that records Amazon Web Services API calls for your Amazon Web Services account and delivers log files to an Amazon S3 bucket. By using information that's collected by Amazon Web Services CloudTrail, you can determine the requests successfully made to Secrets Manager, who made the request, when it was made, and so on. For more about Amazon Web Services Secrets Manager and support for Amazon Web Services CloudTrail, see [Logging Amazon Web Services Secrets Manager Events with Amazon Web Services CloudTrail](https://docs.aws.amazon.com/secretsmanager/latest/userguide/monitoring.html#monitoring_cloudtrail) in the _Amazon Web Services Secrets Manager User Guide_. To learn more about CloudTrail, including enabling it and find your log files, see the [Amazon Web Services CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/what_is_cloud_trail_top_level.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-secretsmanager` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-secretsmanager = "0.14.0"
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

